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
    uri.set_username(&profile.username.replace('%',"%25"))
        .map_err(|_| "Usuario FTP inválido")?;
    {
        let mut segments=uri.path_segments_mut().map_err(|_|"Ruta FTP inválida")?;segments.clear();
        for component in path.components() {match component {
            Component::Normal(name)=>{segments.push(name.to_str().ok_or("Ruta FTP no UTF-8")?);},
            Component::ParentDir=>{segments.pop();},_=>{},
        }}
    }
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
    let info = operation(
        source.query_info_future(
            "standard::type",
            gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
            PRIORITY,
        ),
        channels,
        lifetime,
    )
    .await?;
    if info.file_type() != gio::FileType::Regular {
        return Err("No se copian enlaces ni archivos especiales".into());
    }
    if !destination.is_native() {
        return copy_file(source, destination, channels, lifetime).await;
    }
    let local = destination.path().ok_or("Destino local sin ruta nativa")?;
    let parent = local
        .parent()
        .ok_or("Destino local inválido")?
        .to_path_buf();
    let temporary = nexuterm::runtime::runtime()
        .spawn_blocking(move || {
            tempfile::Builder::new()
                .prefix(".nexuterm-")
                .tempdir_in(parent)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    let partial = temporary.path().join("content");
    let result = async {
        copy_file(source, gio::File::for_path(&partial), channels, lifetime).await?;
        if channels.cancel.load(Ordering::Relaxed) || channels.stop.load(Ordering::Relaxed) {
            return Err("Transferencia cancelada".into());
        }
        nexuterm::runtime::runtime()
            .spawn(async move { tokio::fs::hard_link(partial, local).await })
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| {
                format!("No se publicó el archivo local (no se sobrescriben destinos): {e}")
            })
    }
    .await;
    let _ = nexuterm::runtime::runtime()
        .spawn_blocking(move || drop(temporary))
        .await;
    result
}
async fn plan_download(
    profile: &Profile,
    remote: PathBuf,
    local: PathBuf,
    channels: &WorkerChannels,
    lifetime: &gio::Cancellable,
) -> Result<Vec<nexuterm::transfer::Planned>, String> {
    let mut stack = vec![(remote, local, 0usize)];
    let mut plan = Vec::new();
    while let Some((remote, local, depth)) = stack.pop() {
        if depth > 32 || plan.len() + stack.len() >= 512 {
            return Err("El árbol supera 512 elementos o 32 niveles".into());
        }
        let source = file(profile, &remote)?;
        let info = operation(
            source.query_info_future(
                "standard::type",
                gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
                PRIORITY,
            ),
            channels,
            lifetime,
        )
        .await?;
        let directory = info.file_type() == gio::FileType::Directory;
        if !directory && info.file_type() != gio::FileType::Regular {
            return Err(
                "El árbol contiene enlaces o archivos especiales; no se ha transferido".into(),
            );
        }
        if depth == 0 && !directory {
            return Err("La carpeta remota cambió de tipo".into());
        }
        plan.push(nexuterm::transfer::Planned {
            upload: false,
            directory,
            local: local.clone(),
            remote: remote.clone(),
        });
        if directory {
            let cursor = operation(
                source.enumerate_children_future(
                    "standard::name,standard::type",
                    gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
                    PRIORITY,
                ),
                channels,
                lifetime,
            )
            .await?;
            loop {
                let entries =
                    operation(cursor.next_files_future(100, PRIORITY), channels, lifetime).await?;
                if entries.is_empty() {
                    break;
                }
                for info in entries {
                    let name = info.name();
                    let target = nexuterm::ssh::join_remote(&remote, &name)?;
                    if plan.len() + stack.len() >= 512 {
                        return Err("El árbol supera 512 elementos".into());
                    }
                    stack.push((target, local.join(&name), depth + 1));
                }
            }
            operation(cursor.close_future(PRIORITY), channels, lifetime).await?;
        }
    }
    Ok(plan)
}

async fn require_absent(
    profile: &Profile,
    path: &Path,
    channels: &WorkerChannels,
    lifetime: &gio::Cancellable,
) -> Result<(), String> {
    if path.file_name().is_some_and(|n|n.to_string_lossy().contains('\\')) {return Err("El nombre FTP no puede contener separadores inversos".into());}
    let parent = path.parent().ok_or("Destino FTP inválido")?;
    let name = path.file_name().ok_or("Destino FTP sin nombre")?;
    let cursor = operation(
        file(profile, parent)?.enumerate_children_future(
            "standard::name",
            gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
            PRIORITY,
        ),
        channels,
        lifetime,
    )
    .await?;
    let result = async {
        let mut count = 0;
        loop {
            let entries =
                operation(cursor.next_files_future(100, PRIORITY), channels, lifetime).await?;
            if entries.is_empty() {
                return Ok(());
            }
            count += entries.len();
            if count > 100000 {
                return Err(
                    "La carpeta FTP es demasiado grande para verificar el destino con seguridad"
                        .into(),
                );
            }
            if entries.iter().any(|info| info.name() == Path::new(name)) {
                return Err("El destino ya existe; no se sobrescribe".into());
            }
        }
    }
    .await;
    let close = close_stream(cursor.close_future(PRIORITY)).await;
    result.and(close)
}
async fn close_stream(future: impl Future<Output = Result<(), glib::Error>>) -> Result<(), String> {
    match select(
        Box::pin(future),
        Box::pin(glib::timeout_future(Duration::from_secs(30))),
    )
    .await
    {
        Either::Left((result, _)) => result.map_err(|e| e.to_string()),
        Either::Right(_) => Err("FTP: cierre sin respuesta".into()),
    }
}
async fn copy_file(
    source: gio::File,
    destination: gio::File,
    channels: &WorkerChannels,
    lifetime: &gio::Cancellable,
) -> Result<(), String> {
    // Bound download memory independently of the size reported by the server.
    // This destination is native and private; remote uploads use the curl path.
    let exists = operation(
        async {
            match destination
                .query_info_future(
                    "standard::type",
                    gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
                    PRIORITY,
                )
                .await
            {
                Ok(_) => Ok(true),
                Err(error) if error.matches(gio::IOErrorEnum::NotFound) => Ok(false),
                Err(error) => Err(error),
            }
        },
        channels,
        lifetime,
    )
    .await?;
    if exists {
        return Err("El destino ya existe; no se sobrescribe".into());
    }
    let metadata = operation(
        source.query_info_future(
            "standard::size",
            gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
            PRIORITY,
        ),
        channels,
        lifetime,
    )
    .await?;
    let total = metadata
        .has_attribute("standard::size")
        .then(|| metadata.size().max(0) as u64);
    let input = operation(source.read_future(PRIORITY), channels, lifetime).await?;
    let output = match operation(
        destination.create_future(gio::FileCreateFlags::PRIVATE, PRIORITY),
        channels,
        lifetime,
    )
    .await
    {
        Ok(output) => output,
        Err(error) => {
            let _ = close_stream(input.close_future(PRIORITY)).await;
            return Err(error);
        }
    };
    let mut bytes = 0u64;
    channels.progress.send_replace(Progress { bytes, total });
    let result = async {
        loop {
            let chunk = operation(
                input.read_bytes_future(64 * 1024, PRIORITY),
                channels,
                lifetime,
            )
            .await?;
            if chunk.is_empty() {
                break;
            }
            let expected = chunk.len();
            let writing = async {
                match output.write_all_future(chunk, PRIORITY).await {
                    Ok((_, n, None)) => Ok(n),
                    Ok((_, _, Some(error))) => Err(error),
                    Err((_, error)) => Err(error),
                }
            };
            let written = operation(writing, channels, lifetime).await?;
            if written != expected {
                return Err("FTP: escritura incompleta".into());
            }
            bytes += written as u64;
            channels.progress.send_replace(Progress { bytes, total });
        }
        if total.is_some_and(|total| total != bytes) {
            return Err("El archivo cambió de tamaño durante la transferencia".into());
        }
        Ok(())
    }
    .await;
    let close_output = close_stream(output.close_future(PRIORITY)).await;
    let close_input = close_stream(input.close_future(PRIORITY)).await;
    result.and(close_output).and(close_input)
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
            let transfer = matches!(
                request,
                Request::Upload { .. }
                    | Request::Download { .. }
                    | Request::PlanUpload { .. }
                    | Request::PlanDownload { .. }
                    | Request::CreateLocalDirectory(_)
            );
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
                        Request::PlanUpload { .. }
                        | Request::PlanDownload { .. }
                        | Request::CreateLocalDirectory(_) => Err(
                            "Este servidor FTP heredado no admite transferencias de carpetas"
                                .into(),
                        ),
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
                    Request::PlanUpload { local, directory } => {
                        let cancel = channels.cancel.clone();
                        let stop = channels.stop.clone();
                        let plan = nexuterm::runtime::runtime()
                            .spawn_blocking(move || {
                                nexuterm::transfer::plan_upload(local, directory, &cancel, &stop)
                            })
                            .await
                            .map_err(|e| e.to_string())??;
                        if plan.iter().any(|p|p.remote.components().any(|c|matches!(c,Component::Normal(name) if name.to_string_lossy().contains('\\')))){return Err("El árbol FTP contiene separadores inversos en los nombres".into());}
                        Ok(Event::Planned(plan))
                    }
                    Request::PlanDownload { remote, local } => Ok(Event::Planned(
                        plan_download(&profile, remote, local, &channels, &lifetime).await?,
                    )),
                    Request::CreateLocalDirectory(path) => {
                        let result = nexuterm::runtime::runtime()
                            .spawn(async move { tokio::fs::create_dir(path).await })
                            .await
                            .map_err(|e| e.to_string())?
                            .map_err(|e| e.to_string());
                        Ok(Event::Finished(result))
                    }
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
                            let empty = nexuterm::runtime::runtime()
                                .spawn_blocking(tempfile::NamedTempFile::new)
                                .await
                                .map_err(|e| e.to_string())?
                                .map_err(|e| e.to_string())?;
                            let result = async {
                                require_absent(&profile, &path, &channels, &lifetime).await?;
                                crate::ftp_legacy::upload_file(
                                    &profile,
                                    &legacy_password,
                                    empty.path().to_path_buf(),
                                    path,
                                    &channels,
                                    &lifetime,
                                )
                                .await
                            }
                            .await;
                            let _ = nexuterm::runtime::runtime()
                                .spawn_blocking(move || drop(empty))
                                .await;
                            result
                        };
                        Ok(Event::Finished(result))
                    }
                    Request::Upload { local, directory } => {
                        let name = local.file_name().ok_or("Archivo local inválido")?;
                        let destination = nexuterm::ssh::join_remote(&directory, Path::new(name))?;
                        let result = async {
                            require_absent(&profile, &destination, &channels, &lifetime).await?;
                            crate::ftp_legacy::upload_file(
                                &profile,
                                &legacy_password,
                                local,
                                destination,
                                &channels,
                                &lifetime,
                            )
                            .await
                        }
                        .await;
                        Ok(Event::Finished(result))
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
