//! Graphical FTP through GIO/GVfs, with a Windows CE compatibility transport.
use crate::ui::State;
use futures_util::future::{Either, select};
use gtk::{gio, glib, prelude::*};
use nexuterm::{
    config::Profile,
    transfer::{Client, Entry, Event, PAGE_SIZE, Progress, Request, WorkerChannels},
};
use std::{
    future::Future,
    path::{Component, Path, PathBuf},
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

struct SharedMount {
    users: usize,
    mount: gio::Mount,
    owned: bool,
}
thread_local! { static MOUNTS: std::cell::RefCell<std::collections::HashMap<String,SharedMount>> = std::cell::RefCell::new(std::collections::HashMap::new()); }
fn retain_mount(key: &str, mount: gio::Mount, owned: bool) {
    MOUNTS.with_borrow_mut(|mounts| {
        let shared = mounts.entry(key.into()).or_insert(SharedMount {
            users: 0,
            mount,
            owned,
        });
        shared.users += 1;
        shared.owned |= owned;
    });
}
fn release_mount(key: &str) -> Option<gio::Mount> {
    MOUNTS.with_borrow_mut(|mounts| {
        let shared = mounts.get_mut(key)?;
        shared.users -= 1;
        if shared.users != 0 {
            return None;
        }
        let shared = mounts.remove(key)?;
        shared.owned.then_some(shared.mount)
    })
}

const ATTRIBUTES: &str = "standard::name,standard::type,standard::size,unix::mode";
const PRIORITY: glib::Priority = glib::Priority::DEFAULT;

fn file(profile: &Profile, path: &Path) -> Result<gio::File, String> {
    let host = if profile.host.contains(':') {
        format!("[{}]", profile.host)
    } else {
        profile.host.clone()
    };
    let mut uri =
        url::Url::parse(&format!("ftp://{host}:{}/", profile.port)).map_err(|e| e.to_string())?;
    uri.set_username(&profile.username)
        .map_err(|_| "Usuario FTP inválido")?;
    let path = path.to_str().ok_or("Ruta FTP no UTF-8")?;
    uri.set_path(if path == "." { "/" } else { path });
    Ok(gio::File::for_uri(uri.as_str()))
}
// gio-rs does not expose this async GFile method in the pinned version.
async fn enclosing_mount(file: &gio::File) -> Result<gio::Mount, glib::Error> {
    use glib::translate::{ToGlibPtr, from_glib_full};
    gio::GioFuture::new(file, |file, cancellable, send| {
        type Reply = gio::GioFutureResult<Result<gio::Mount, glib::Error>>;
        unsafe extern "C" fn done(
            source: *mut glib::gobject_ffi::GObject,
            result: *mut gio::ffi::GAsyncResult,
            data: glib::ffi::gpointer,
        ) {
            unsafe {
                let send = Box::from_raw(data as *mut Reply);
                let mut error = std::ptr::null_mut();
                let mount =
                    gio::ffi::g_file_find_enclosing_mount_finish(source.cast(), result, &mut error);
                send.resolve(if error.is_null() {
                    Ok(from_glib_full(mount))
                } else {
                    Err(from_glib_full(error))
                });
            }
        }
        unsafe {
            gio::ffi::g_file_find_enclosing_mount_async(
                file.to_glib_none().0,
                0,
                cancellable.to_glib_none().0,
                Some(done),
                Box::into_raw(Box::new(send)).cast(),
            );
        }
    })
    .await
}

async fn stopped(stop: &AtomicBool, lifetime: &gio::Cancellable) {
    while !stop.load(Ordering::Relaxed) && !lifetime.is_cancelled() {
        glib::timeout_future(Duration::from_millis(50)).await;
    }
}
async fn operation<T>(
    future: impl Future<Output = Result<T, glib::Error>>,
    channels: &WorkerChannels,
    lifetime: &gio::Cancellable,
) -> Result<T, String> {
    let interrupt = async {
        let started = Instant::now();
        while !channels.stop.load(Ordering::Relaxed)
            && !channels.cancel.load(Ordering::Relaxed)
            && !lifetime.is_cancelled()
            && started.elapsed() < Duration::from_secs(30)
        {
            glib::timeout_future(Duration::from_millis(50)).await;
        }
    };
    match select(Box::pin(future), Box::pin(interrupt)).await {
        Either::Left((result, _)) => result.map_err(|e| e.to_string()),
        Either::Right(_) => Err("Operación FTP cancelada o tiempo de espera agotado".into()),
    }
}
async fn page(
    cursor: &gio::FileEnumerator,
    directory: &Path,
    channels: &WorkerChannels,
    lifetime: &gio::Cancellable,
) -> Result<Event, String> {
    let infos = operation(
        cursor.next_files_future(PAGE_SIZE as i32, PRIORITY),
        channels,
        lifetime,
    )
    .await?;
    let more = infos.len() == PAGE_SIZE;
    let mut entries = Vec::with_capacity(infos.len());
    for info in infos {
        let name = info.name();
        let mut parts = name.components();
        if !matches!(parts.next(), Some(Component::Normal(_))) || parts.next().is_some() {
            return Err("El servidor FTP envió un nombre de archivo inválido".into());
        }
        entries.push(Entry {
            name,
            directory: info.file_type() == gio::FileType::Directory,
            regular: info.file_type() == gio::FileType::Regular,
            size: u64::try_from(info.size()).ok(),
            permissions: info
                .has_attribute("unix::mode")
                .then(|| info.attribute_uint32("unix::mode")),
        });
    }
    Ok(Event::Page {
        directory: directory.into(),
        entries,
        more,
    })
}
async fn copy(
    source: gio::File,
    destination: gio::File,
    channels: &WorkerChannels,
    lifetime: &gio::Cancellable,
) -> Result<(), String> {
    let cancel = gio::Cancellable::new();
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let progress = channels.progress.clone();
    progress.send_replace(Progress::default());
    source.copy_async(
        &destination,
        gio::FileCopyFlags::NOFOLLOW_SYMLINKS,
        PRIORITY,
        Some(&cancel),
        Some(Box::new(move |bytes, total| {
            progress.send_replace(Progress {
                bytes: bytes.max(0) as u64,
                total: u64::try_from(total).ok(),
            });
        })),
        move |result| {
            let _ = sender.send(result);
        },
    );
    let monitor = async {
        loop {
            if channels.cancel.load(Ordering::Relaxed)
                || channels.stop.load(Ordering::Relaxed)
                || lifetime.is_cancelled()
            {
                cancel.cancel();
            }
            glib::timeout_future(Duration::from_millis(50)).await;
        }
    };
    match select(Box::pin(receiver), Box::pin(monitor)).await {
        Either::Left((result, _)) => result
            .map_err(|_| "Se cerró la operación FTP".to_string())?
            .map_err(|e| e.to_string()),
        Either::Right(_) => unreachable!(),
    }
}
pub fn start(state: &Rc<State>, profile: Profile, password: zeroize::Zeroizing<String>) -> Client {
    let (client, mut channels) = Client::channels();
    let lifetime = gio::Cancellable::new();
    state.operations.borrow_mut().push(lifetime.downgrade());
    glib::spawn_future_local(async move {
        let root = match file(&profile, Path::new("/")) {
            Ok(file) => file,
            Err(error) => {
                let _ = channels.events.send(Event::Error(error)).await;
                return;
            }
        };
        let mount_operation = gio::MountOperation::new();
        let legacy_password = password.clone();
        let username = profile.username.clone();
        let attempts = std::cell::Cell::new(0);
        mount_operation.connect_ask_password(move |op, _, _, _, _| {
            attempts.set(attempts.get() + 1);
            if attempts.get() > 1 {
                op.reply(gio::MountOperationResult::Aborted);
                return;
            }
            op.set_username(Some(&username));
            op.set_password(Some(&password));
            op.set_password_save(gio::PasswordSave::Never);
            op.reply(gio::MountOperationResult::Handled);
        });
        let mut legacy_mode = false;
        let mut legacy_listing: Option<crate::ftp_legacy::Listing> = None;
        let mut mounted = false;
        let mut mount_key = None;
        let mut cursor = None;
        let mut directory = PathBuf::from("/");
        loop {
            let request = match select(
                Box::pin(channels.requests.recv()),
                Box::pin(stopped(&channels.stop, &lifetime)),
            )
            .await
            {
                Either::Left((Some(request), _)) => request,
                _ => break,
            };
            let transfer = matches!(request, Request::Upload { .. } | Request::Download { .. });
            let result: Result<Event, String> = async {
                if !mounted {
                    let result = root.mount_enclosing_volume_future(
                        gio::MountMountFlags::NONE,
                        Some(&mount_operation),
                    );
                    // AlreadyMounted is an existing shared GVfs mount; never unmount it.
                    let created = match operation(result, &channels, &lifetime).await {
                        Ok(()) => true,
                        Err(error) => {
                            let probe =
                                operation(enclosing_mount(&root), &channels, &lifetime).await;
                            if probe.is_err() {
                                return Err(format!(
                                    "FTP: {error}. Comprueba que gvfs-backends esté instalado."
                                ));
                            }
                            false
                        }
                    };
                    let mount = operation(enclosing_mount(&root), &channels, &lifetime).await?;
                    let key = root.uri().to_string();
                    retain_mount(&key, mount, created);
                    mount_key = Some(key);
                    mounted = true;
                }
                if legacy_mode {
                    return match request {
                        Request::Navigate(path) => {
                            let path = if path == Path::new(".") {
                                PathBuf::from("/")
                            } else {
                                path
                            };
                            let mut listing = crate::ftp_legacy::list(
                                &profile,
                                &legacy_password,
                                &path,
                                &channels,
                                &lifetime,
                            )
                            .await?;
                            let event = listing.page();
                            legacy_listing = Some(listing);
                            Ok(event)
                        }
                        Request::Next => Ok(legacy_listing
                            .as_mut()
                            .ok_or("Abre una carpeta primero")?
                            .page()),
                        transfer => Ok(Event::Finished(
                            crate::ftp_legacy::transfer(
                                &profile,
                                &legacy_password,
                                transfer,
                                &channels,
                                &lifetime,
                            )
                            .await,
                        )),
                    };
                }
                match request {
                    Request::Navigate(path) => {
                        let current = file(&profile, &path)?;
                        cursor = Some(
                            operation(
                                current.enumerate_children_future(
                                    ATTRIBUTES,
                                    gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
                                    PRIORITY,
                                ),
                                &channels,
                                &lifetime,
                            )
                            .await?,
                        );
                        // get_relative_path decodes the URI without exposing credentials.
                        directory = PathBuf::from("/")
                            .join(root.relative_path(&current).unwrap_or_default());
                        let event =
                            page(cursor.as_ref().unwrap(), &directory, &channels, &lifetime)
                                .await?;
                        if matches!(&event, Event::Page { entries, .. } if entries.is_empty()) {
                            let mut listing = crate::ftp_legacy::list(
                                &profile,
                                &legacy_password,
                                &directory,
                                &channels,
                                &lifetime,
                            )
                            .await?;
                            if !listing.is_empty() {
                                legacy_mode = true;
                                let event = listing.page();
                                legacy_listing = Some(listing);
                                return Ok(event);
                            }
                        }
                        Ok(event)
                    }
                    Request::Next => {
                        page(
                            cursor.as_ref().ok_or("Abre una carpeta primero")?,
                            &directory,
                            &channels,
                            &lifetime,
                        )
                        .await
                    }
                    Request::Create {
                        directory,
                        name,
                        folder,
                    } => {
                        let path = nexuterm::ssh::join_remote(&directory, &name)?;
                        let destination = file(&profile, &path)?;
                        let result = if folder {
                            operation(
                                destination.make_directory_future(PRIORITY),
                                &channels,
                                &lifetime,
                            )
                            .await
                        } else {
                            match operation(
                                destination.create_future(gio::FileCreateFlags::NONE, PRIORITY),
                                &channels,
                                &lifetime,
                            )
                            .await
                            {
                                Ok(stream) => {
                                    operation(stream.close_future(PRIORITY), &channels, &lifetime)
                                        .await
                                        .map(|_| ())
                                }
                                Err(error) => Err(error),
                            }
                        };
                        Ok(Event::Finished(result))
                    }
                    Request::Upload { local, directory } => {
                        let name = local.file_name().ok_or("Archivo local inválido")?;
                        let destination = file(&profile, &directory.join(name))?;
                        Ok(Event::Finished(
                            copy(
                                gio::File::for_path(local),
                                destination,
                                &channels,
                                &lifetime,
                            )
                            .await,
                        ))
                    }
                    Request::Download { remote, local } => Ok(Event::Finished(
                        copy(
                            file(&profile, &remote)?,
                            gio::File::for_path(local),
                            &channels,
                            &lifetime,
                        )
                        .await,
                    )),
                }
            }
            .await;
            if channels.stop.load(Ordering::Relaxed) || lifetime.is_cancelled() {
                break;
            }
            let event = match result {
                Ok(event) => event,
                Err(error) if transfer => Event::Finished(Err(error)),
                Err(error) => Event::Error(error),
            };
            if channels.events.send(event).await.is_err() {
                break;
            }
        }
        if let Some(cursor) = cursor {
            let _ = cursor.close_future(PRIORITY).await;
        }
        if let Some(mount) = mount_key.and_then(|key| release_mount(&key)) {
            let unmount = mount.unmount_with_operation_future(
                gio::MountUnmountFlags::NONE,
                gio::MountOperation::NONE,
            );
            let _ = select(
                Box::pin(unmount),
                Box::pin(glib::timeout_future(Duration::from_secs(3))),
            )
            .await;
        }
        #[cfg(debug_assertions)]
        if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
            && std::env::var_os("NEXUTERM_SMOKE_FILE_BROWSER").is_some()
        {
            eprintln!("NEXUTERM_SMOKE_FTP_WORKER_STOPPED");
        }
    });
    client
}
