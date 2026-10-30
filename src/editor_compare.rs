//! Explicit manual conflict review. Applying changes never writes the original.
use crate::recovery_ui::{Tracker, snapshot};
use gtk::{gio, glib, prelude::*};
use nexuterm::{
    remote_edit::{Comparison, RemoteDocument},
    text_diff,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};
struct Review {
    base: Option<String>,
    server: String,
    etag: Option<String>,
    ssh: Option<Comparison>,
}
#[allow(clippy::too_many_arguments)]
pub fn install(
    toolbar: &gtk::Box,
    window: &gtk::ApplicationWindow,
    buffer: &gtk::TextBuffer,
    status: &gtk::Label,
    save: &gtk::Button,
    loaded: Rc<Cell<bool>>,
    busy: Rc<Cell<bool>>,
    tag: Rc<RefCell<Option<String>>>,
    file: Rc<RefCell<gio::File>>,
    remote: Rc<RefCell<Option<Arc<RemoteDocument>>>>,
    recovery: Rc<Tracker>,
) {
    let button = gtk::Button::with_label("Comparar versiones…");
    button.set_widget_name("editor-compare");
    toolbar.append(&button);
    let window = window.downgrade();
    let buffer = buffer.downgrade();
    let status = status.downgrade();
    let save = save.downgrade();
    #[cfg(debug_assertions)]
    if std::env::var_os("NEXUTERM_SMOKE_EDITOR_COMPARE").is_some() {
        smoke::run(
            window.clone(),
            buffer.clone(),
            status.clone(),
            save.clone(),
            button.clone(),
            loaded.clone(),
            busy.clone(),
            recovery.clone(),
        );
    }
    button.connect_clicked(move |_|{
        if busy.get() || !loaded.get(){return;}
        let (Some(window),Some(buffer),Some(status))=(window.upgrade(),buffer.upgrade(),status.upgrade()) else{return;};
        let local=match snapshot(&buffer){Ok(v)=>v,Err(e)=>{status.set_text(&e);return;}};
        busy.set(true);if let Some(save)=save.upgrade(){save.set_sensitive(false);}
        status.set_text("Leyendo la versión actual para comparar; no se guarda ni se sobrescribe nada…");
        let doc=remote.borrow().clone();let file=file.borrow().clone();let recovery=recovery.clone();
        let busy=busy.clone();let tag=tag.clone();let save=save.clone();
        glib::spawn_future_local(async move{
            let result=async{
                let review=if let Some(doc)=&doc{
                    let document=doc.clone();
                    let comparison=nexuterm::runtime::runtime().spawn(async move{document.comparison().await}).await.map_err(|e|e.to_string())??;
                    Review{base:comparison.base.clone(),server:comparison.remote.clone(),etag:None,ssh:Some(comparison)}
                }else{
                    if matches!(recovery.source(),nexuterm::recovery::Source::Remote{..}){return Err("Vincula primero el borrador al mismo perfil SSH para comparar su original".into());}
                    let (server,etag)=crate::files::read_text(&file).await?;
                    Review{base:recovery.base(),server,etag,ssh:None}
                };
                let a=review.base.clone();let b=local.clone();let c=review.server.clone();
                let diffs=nexuterm::runtime::runtime().spawn_blocking(move||a.map(|a|(text_diff::compare(&a,&b),text_diff::compare(&a,&c)))).await.map_err(|e|e.to_string())?;
                Ok::<_,String>((review,diffs))
            }.await;
            match result{
                Ok((review,diffs))=>show(&window,&buffer,&status,save,busy,tag,file,doc,recovery,local,review,diffs),
                Err(e)=>{status.set_text(&format!("No se pudo comparar: {e}. El borrador se conserva."));busy.set(false);if let Some(save)=save.upgrade(){save.set_sensitive(tag.borrow().is_some() && !doc.as_ref().is_some_and(|d|d.requires_review()));}}
            }
        });
    });
}
fn pane(title: &str, text: &str, editable: bool) -> (gtk::Box, gtk::TextBuffer) {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let label = gtk::Label::new(Some(title));
    label.set_xalign(0.0);
    label.add_css_class("heading");
    root.append(&label);
    let view = gtk::TextView::builder()
        .editable(editable)
        .cursor_visible(editable)
        .monospace(true)
        .wrap_mode(gtk::WrapMode::None)
        .left_margin(8)
        .right_margin(8)
        .top_margin(6)
        .bottom_margin(6)
        .build();
    let buffer = view.buffer();
    buffer.set_text(text);
    root.append(
        &gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&view)
            .build(),
    );
    (root, buffer)
}
fn metadata_summary(title: &str, m: openssh_sftp_client::metadata::MetaData) -> String {
    let id = |id: Option<u32>| id.map(|v| v.to_string()).unwrap_or_else(|| "—".into());
    let mode = m
        .permissions()
        .map(|p| format!("{:04o}", p.as_raw().bits()))
        .unwrap_or_else(|| "—".into());
    let modified = m
        .modified()
        .and_then(|t| glib::DateTime::from_unix_local(t.into_raw() as i64).ok())
        .and_then(|t| t.format("%d/%m/%Y %H:%M:%S").ok())
        .map(|t| t.to_string())
        .unwrap_or_else(|| "—".into());
    format!(
        "{title} · {} · propietario {} · grupo {} · permisos {mode} · modificado {modified}",
        m.len()
            .map(|v| format!("{v} bytes"))
            .unwrap_or_else(|| "tamaño no informado".into()),
        id(m.uid()),
        id(m.gid())
    )
}
fn highlight(buffer: &gtk::TextBuffer, ranges: impl Iterator<Item = std::ops::Range<usize>>) {
    let color = if adw::StyleManager::default().is_dark() {
        "#574129"
    } else {
        "#ffe3a3"
    };
    let tag = gtk::TextTag::builder().background(color).build();
    buffer.tag_table().add(&tag);
    for range in ranges {
        if range.is_empty() {
            continue;
        }
        let start = buffer
            .iter_at_line(range.start as i32)
            .unwrap_or_else(|| buffer.end_iter());
        let end = buffer
            .iter_at_line(range.end as i32)
            .unwrap_or_else(|| buffer.end_iter());
        buffer.apply_tag(&tag, &start, &end);
    }
}
#[allow(clippy::too_many_arguments)]
fn show(
    parent: &gtk::ApplicationWindow,
    original: &gtk::TextBuffer,
    status: &gtk::Label,
    save: glib::WeakRef<gtk::Button>,
    busy: Rc<Cell<bool>>,
    tag: Rc<RefCell<Option<String>>>,
    file: gio::File,
    doc: Option<Arc<RemoteDocument>>,
    recovery: Rc<Tracker>,
    local: String,
    review: Review,
    diffs: Option<(text_diff::Diff, text_diff::Diff)>,
) {
    let dialog = gtk::Window::builder()
        .title("Comparar y resolver versiones")
        .transient_for(parent)
        .modal(true)
        .destroy_with_parent(true)
        .default_width(1100)
        .default_height(760)
        .build();
    dialog.set_widget_name("editor-comparison");
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    for set in [
        gtk::Widget::set_margin_top,
        gtk::Widget::set_margin_bottom,
        gtk::Widget::set_margin_start,
        gtk::Widget::set_margin_end,
    ] {
        set(body.upcast_ref(), 12);
    }
    let notice = gtk::Label::new(Some(
        "Base: texto al abrir o guardar. Local: tu borrador. Original actual: última lectura del destino. Las zonas resaltadas difieren de la base. Edita el resultado o elige una versión completa. Aplicar solo cambia el borrador; Guardar comprueba de nuevo el original.",
    ));
    notice.set_wrap(true);
    notice.set_xalign(0.0);
    body.append(&notice);
    let (base_pane,base_buffer)=pane(if review.base.is_some(){"Base"}else{"Base no disponible"},review.base.as_deref().unwrap_or("Este borrador antiguo no conserva el texto base. Compara Local y Original actual sin reconstruir una base."),false);
    base_pane.set_widget_name("editor-compare-base");
    let (local_pane, local_buffer) = pane("Local · sin guardar", &local, false);
    local_pane.set_widget_name("editor-compare-local");
    let (server_pane, server_buffer) = pane("Original actual", &review.server, false);
    server_pane.set_widget_name("editor-compare-remote");
    let right = gtk::Paned::new(gtk::Orientation::Horizontal);
    right.set_start_child(Some(&local_pane));
    right.set_end_child(Some(&server_pane));
    right.set_position(350);
    let versions = gtk::Paned::new(gtk::Orientation::Horizontal);
    versions.set_start_child(Some(&base_pane));
    versions.set_end_child(Some(&right));
    versions.set_position(350);
    if let Some((a, b)) = &diffs {
        highlight(
            &base_buffer,
            a.changes
                .iter()
                .chain(b.changes.iter())
                .map(|c| c.base.clone()),
        );
        highlight(&local_buffer, a.changes.iter().map(|c| c.edited.clone()));
        highlight(&server_buffer, b.changes.iter().map(|c| c.edited.clone()));
        if !a.detailed || !b.detailed {
            notice.set_text("Comparación extensa: se resaltan regiones completas para acotar memoria y CPU. Las tres versiones están completas; revisa manualmente el resultado. Aplicar solo cambia el borrador, no guarda el original.");
        }
    }
    if let Some(comparison) = &review.ssh {
        for (title, m) in [
            ("Metadatos base", comparison.base_metadata),
            ("Original actual", Some(comparison.metadata)),
        ] {
            if let Some(m) = m {
                let label = gtk::Label::new(Some(&metadata_summary(title, m)));
                label.set_xalign(0.0);
                label.set_wrap(true);
                body.append(&label);
            }
        }
        let limits = gtk::Label::new(Some(
            "Se conservan propietario, grupo y permisos POSIX. ACL y atributos extendidos no están disponibles en este transporte.",
        ));
        limits.set_xalign(0.0);
        limits.set_wrap(true);
        limits.add_css_class("dim-label");
        body.append(&limits);
    }
    let (result_pane, result) = pane("Resultado · editable", &local, true);
    result_pane.set_widget_name("editor-merge-result");
    let split = gtk::Paned::new(gtk::Orientation::Vertical);
    split.set_vexpand(true);
    split.set_start_child(Some(&versions));
    split.set_end_child(Some(&result_pane));
    split.set_position(330);
    body.append(&split);
    let checked =
        gtk::CheckButton::with_label("He revisado el resultado y quiero aplicarlo al borrador");
    checked.set_widget_name("editor-merge-reviewed");
    body.append(&checked);
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let keep_local = gtk::Button::with_label("Usar todo Local");
    keep_local.set_widget_name("editor-merge-use-local");
    let keep_server = gtk::Button::with_label("Usar todo Original actual");
    keep_server.set_widget_name("editor-merge-use-remote");
    let cancel = gtk::Button::with_label("Cancelar");
    cancel.set_widget_name("editor-merge-cancel");
    let apply = gtk::Button::with_label("Aplicar al borrador");
    apply.set_widget_name("editor-merge-apply");
    apply.add_css_class("suggested-action");
    apply.set_sensitive(false);
    for button in [&keep_local, &keep_server, &cancel, &apply] {
        actions.append(button);
    }
    body.append(&actions);
    let hint = gtk::Label::new(None);
    hint.set_widget_name("editor-merge-status");
    hint.set_wrap(true);
    hint.set_xalign(0.0);
    body.append(&hint);
    let review = Rc::new(review);
    let applying = Rc::new(Cell::new(false));
    let confirm = checked.downgrade();
    result.connect_changed(move |_| {
        if let Some(c) = confirm.upgrade() {
            c.set_active(false);
        }
    });
    for (button, text) in [
        (&keep_local, local.clone()),
        (&keep_server, review.server.clone()),
    ] {
        let result = result.clone();
        let checked = checked.clone();
        button.connect_clicked(move |_| {
            result.set_text(&text);
            checked.set_active(true);
        });
    }
    let ready = apply.downgrade();
    let working = applying.clone();
    checked.connect_toggled(move |b| {
        if let Some(apply) = ready.upgrade() {
            apply.set_sensitive(b.is_active() && !working.get());
        }
    });
    let reset_busy = busy.clone();
    let reset_save = save.clone();
    let reset_tag = tag.clone();
    let reset_doc = doc.clone();
    let working = applying.clone();
    dialog.connect_close_request(move |_| {
        if working.get() {
            return glib::Propagation::Stop;
        }
        reset_busy.set(false);
        if let Some(save) = reset_save.upgrade() {
            save.set_sensitive(
                reset_tag.borrow().is_some()
                    && !reset_doc.as_ref().is_some_and(|d| d.requires_review()),
            );
        }
        glib::Propagation::Proceed
    });
    let close = dialog.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(d) = close.upgrade() {
            d.close();
        }
    });
    let original = original.downgrade();
    let status = status.downgrade();
    let weak = dialog.downgrade();
    let cancel = cancel.downgrade();
    let result = result.downgrade();
    let hint = hint.downgrade();
    let confirmation=checked.downgrade();
    apply.connect_clicked(move |button|{
        if !confirmation.upgrade().is_some_and(|c|c.is_active()) || applying.replace(true){return;}
        let (Some(original),Some(result))=(original.upgrade(),result.upgrade())else{applying.set(false);return;};
        let text=match snapshot(&result){Ok(v)=>v,Err(e)=>{if let Some(hint)=hint.upgrade(){hint.set_text(&e);}applying.set(false);return;}};
        if snapshot(&original).ok().as_deref()!=Some(local.as_str()){if let Some(hint)=hint.upgrade(){hint.set_text("El borrador cambió durante la comparación. Cancela y abre una comparación nueva.");}applying.set(false);return;}
        button.set_sensitive(false);if let Some(c)=cancel.upgrade(){c.set_sensitive(false);}
        let expected_local=local.clone();
        let checked=confirmation.clone();
        let (review,doc,file,tag,recovery,working,weak,status,hint,cancel,button)=(review.clone(),doc.clone(),file.clone(),tag.clone(),recovery.clone(),applying.clone(),weak.clone(),status.clone(),hint.clone(),cancel.clone(),button.downgrade());
        glib::spawn_future_local(async move{
            let outcome=async{
                if let Some(doc)=&doc{let document=doc.clone();let comparison=review.ssh.clone().ok_or("Falta la versión SSH")?;nexuterm::runtime::runtime().spawn(async move{document.accept_comparison(&comparison).await}).await.map_err(|e|e.to_string())??;}
                else{let (current,current_tag)=crate::files::read_text(&file).await?;if current_tag.is_none() || current_tag!=review.etag || current!=review.server{return Err("Conflicto: el original volvió a cambiar o no ofrece una versión segura; cancela y compara de nuevo".into());}*tag.borrow_mut()=current_tag;}
                recovery.set_base(Some(review.server.clone()));if let Some(doc)=&doc{recovery.set_source(doc.recovery_source());}
                if snapshot(&original).ok().as_deref()!=Some(expected_local.as_str()){if let Some(doc)=&doc{doc.require_review();}else{*tag.borrow_mut()=None;}return Err("La base se ha verificado, pero el borrador cambió mientras se comparaba; se conserva sin reemplazar. Abre otra comparación.".into());}
                if snapshot(&result).ok().as_deref()!=Some(text.as_str()) || !checked.upgrade().is_some_and(|c|c.is_active()){if let Some(doc)=&doc{doc.require_review();}else{*tag.borrow_mut()=None;}return Err("El resultado cambió durante la comprobación. Se conserva sin aplicar; cancela y compara de nuevo.".into());}
                original.begin_user_action();original.set_text(&text);original.end_user_action();original.set_modified(true);
                recovery.flush().await?;
                Ok::<_,String>(())
            }.await;
            working.set(false);if let Some(c)=cancel.upgrade(){c.set_sensitive(true);}
            match outcome{
                Ok(())=>{if let Some(status)=status.upgrade(){status.set_text("Resolución aplicada al borrador. Pulsa Guardar para publicar con una nueva comprobación de versión.");}if let Some(d)=weak.upgrade(){d.close();}}
                Err(e)=>{if let Some(hint)=hint.upgrade(){hint.set_text(&format!("No se confirmó la resolución: {e}. No se ha guardado el original."));}if let Some(b)=button.upgrade(){b.set_sensitive(checked.upgrade().is_some_and(|c|c.is_active()));}}
            }
        });
    });
    dialog.set_child(Some(&body));
    dialog.present();
}

#[cfg(debug_assertions)]
#[path = "editor_compare_smoke.rs"]
mod smoke;

#[cfg(debug_assertions)]
pub(crate) async fn smoke_recovered(root: &gtk::Widget, text: &gtk::TextBuffer, tracker: &Tracker) {
    smoke::recovered(root, text, tracker).await;
}
