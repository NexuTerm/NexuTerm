use crate::ui::State;
use adw::prelude::*;
use gtk::{gio, glib};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

const PRIORITY: glib::Priority = glib::Priority::DEFAULT;
const EDIT_LIMIT: usize = 8 * 1024 * 1024;

pub fn choose_editor(state: &Rc<State>, window: &gtk::ApplicationWindow) {
    let dialog = gtk::FileDialog::builder()
        .title("Abrir archivo de texto")
        .build();
    let weak = Rc::downgrade(state);
    let win = window.clone();
    glib::spawn_future_local(async move {
        if let Ok(file) = dialog.open_future(Some(&win)).await
            && let Some(s) = weak.upgrade()
        {
            editor(&s, &win, file);
        }
    });
}

pub fn editor(state: &Rc<State>, window: &gtk::ApplicationWindow, file: gio::File) {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let save = gtk::Button::with_label("Guardar");
    save.set_sensitive(false);
    let status = gtk::Label::new(Some("Cargando texto UTF-8 (máximo 8 MiB)…"));
    status.set_xalign(0.0);
    status.set_wrap(true);
    root.append(&save);
    root.append(&status);
    let text = gtk::TextView::builder()
        .monospace(true)
        .editable(false)
        .left_margin(12)
        .right_margin(12)
        .top_margin(12)
        .build();
    let buffer = text.buffer();
    state
        .documents
        .borrow_mut()
        .retain(|b| b.upgrade().is_some());
    state.documents.borrow_mut().push(buffer.downgrade());
    root.append(
        &gtk::ScrolledWindow::builder()
            .child(&text)
            .vexpand(true)
            .build(),
    );
    let etag = Rc::new(RefCell::new(None::<String>));
    let busy = Rc::new(Cell::new(false));
    let loaded = Rc::new(Cell::new(false));
    state.writes.borrow_mut().retain(|b| b.strong_count() > 0);
    state.writes.borrow_mut().push(Rc::downgrade(&busy));
    let title = file
        .basename()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Editor".into());
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let label = gtk::Label::new(Some(&title));
    header.append(&label);
    let close = gtk::Button::from_icon_name("window-close-symbolic");
    header.append(&close);
    let page = state.notebook.append(&root);
    page.set_title(&title);
    state.notebook.set_selected_page(&page);
    let target = page.downgrade();
    let close_button = close.clone();
    state.notebook.connect_close_page(move |_, page| {
        if target.upgrade().as_ref() == Some(page) {
            close_button.emit_clicked();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    let tab = root.downgrade();
    let nb = state.notebook.downgrade();
    let buf = buffer.downgrade();
    let w = window.downgrade();
    let b = busy.clone();
    let st = status.downgrade();
    close.connect_clicked(move |_| {
        if b.get() {
            if let Some(s) = st.upgrade() {
                s.set_text("Espera a que finalice la escritura antes de cerrar.");
            }
            return;
        }
        if let Some(buffer) = buf.upgrade()
            && buffer.is_modified()
        {
            let Some(window) = w.upgrade() else {
                return;
            };
            let dialog = gtk::AlertDialog::builder()
                .modal(true)
                .message("¿Descartar los cambios sin guardar?")
                .buttons(["Cancelar", "Descartar"])
                .cancel_button(0)
                .default_button(0)
                .build();
            let nb = nb.clone();
            let tab = tab.clone();
            glib::spawn_future_local(async move {
                if dialog.choose_future(Some(&window)).await == Ok(1) {
                    remove_page(&nb, &tab);
                }
            });
            return;
        }
        remove_page(&nb, &tab);
    });
    let file_save = file.clone();
    let buf = buffer.downgrade();
    let tag = etag.clone();
    let b = busy.clone();
    let l = loaded.clone();
    let st = status.downgrade();
    save.connect_clicked(move|button|{
        if b.replace(true)||!l.get(){return;}
        let Some(buffer)=buf.upgrade()else{b.set(false);return;};
        let contents=buffer.text(&buffer.start_iter(),&buffer.end_iter(),true).to_string();
        if contents.len()>EDIT_LIMIT{b.set(false);if let Some(s)=st.upgrade(){s.set_text("El documento supera el límite de 8 MiB");}return;}
        let file=file_save.clone();let expected=tag.borrow().clone();let tag=tag.clone();let b=b.clone();let st=st.clone();let button=button.downgrade();let buf=buffer.downgrade();
        if let Some(button)=button.upgrade(){button.set_sensitive(false);}
        glib::spawn_future_local(async move{
            match file.replace_contents_future(contents.clone().into_bytes(),expected.as_deref(),false,gio::FileCreateFlags::NONE).await {
                Ok((_,new_tag))=>{
                    *tag.borrow_mut()=new_tag.map(|s|s.into());
                    if let Some(buffer)=buf.upgrade() && buffer.text(&buffer.start_iter(),&buffer.end_iter(),true).as_str()==contents{buffer.set_modified(false);}
                    if let Some(s)=st.upgrade(){s.set_text("Guardado. La próxima escritura comprobará la versión remota.");}
                },Err((_,e))=>{if let Some(s)=st.upgrade(){s.set_text(&format!("No se guardó: {e}. Si cambió fuera del editor, vuelve a abrirlo y compara antes de guardar."));}}
            }
            b.set(false);if let Some(button)=button.upgrade(){button.set_sensitive(tag.borrow().is_some());}
        });
    });
    let buf = buffer.downgrade();
    let view = text.downgrade();
    let save = save.downgrade();
    let status = status.downgrade();
    glib::spawn_future_local(async move {
        let result = read_text(&file).await;
        if let (Some(buffer), Some(view), Some(save), Some(status)) = (
            buf.upgrade(),
            view.upgrade(),
            save.upgrade(),
            status.upgrade(),
        ) {
            match result {
                Ok((contents, tag)) => {
                    buffer.set_text(&contents);
                    buffer.set_modified(false);
                    view.set_editable(true);
                    save.set_sensitive(tag.is_some());
                    *etag.borrow_mut() = tag;
                    loaded.set(true);
                    status.set_text(if etag.borrow().is_some(){"UTF-8 · guardado con comprobación de versión"}else{"El servidor no aporta versión/etag: edición disponible, guardado deshabilitado para evitar sobrescrituras."});
                }
                Err(e) => status.set_text(&format!("No se pudo abrir: {e}")),
            }
        }
    });
}

fn remove_page(nb: &glib::WeakRef<adw::TabView>, tab: &glib::WeakRef<gtk::Box>) {
    if let (Some(n), Some(t)) = (nb.upgrade(), tab.upgrade())
        && let Some(i) = n
            .pages()
            .iter::<adw::TabPage>()
            .filter_map(Result::ok)
            .find(|p| p.child() == t)
    {
        n.close_page_finish(&i, true);
    }
}

async fn read_text(file: &gio::File) -> Result<(String, Option<String>), String> {
    let before = file
        .query_info_future("etag::value", gio::FileQueryInfoFlags::NONE, PRIORITY)
        .await
        .map_err(|e| e.to_string())?;
    let stream = file
        .read_future(PRIORITY)
        .await
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    let read_result = async {
        loop {
            let chunk = stream
                .read_bytes_future(65536, PRIORITY)
                .await
                .map_err(|e| e.to_string())?;
            if chunk.is_empty() {
                break;
            }
            if bytes.len() + chunk.len() > EDIT_LIMIT {
                return Err("Archivo mayor de 8 MiB".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok::<(), String>(())
    }
    .await;
    let closed = stream.close_future(PRIORITY).await;
    read_result?;
    closed.map_err(|e| e.to_string())?;
    let after = file
        .query_info_future("etag::value", gio::FileQueryInfoFlags::NONE, PRIORITY)
        .await
        .map_err(|e| e.to_string())?;
    let before = before.etag();
    let after = after.etag();
    if before != after {
        return Err("El archivo cambió durante la lectura; vuelve a abrirlo".into());
    }
    if bytes.contains(&0) {
        return Err("Archivo binario; no se abre como texto".into());
    }
    let contents = String::from_utf8(bytes)
        .map_err(|_| "Codificación no UTF-8: conviértela antes de editar")?;
    Ok((contents, after.map(|s| s.into())))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_utf8_and_conflict_protection() {
        let dir = std::env::temp_dir().join(format!("nexuterm-editor-test-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("document.txt");
        std::fs::write(&path, "áé Linux\r\n").unwrap();
        let file = gio::File::for_path(&path);
        let context = glib::MainContext::new();
        context.block_on(async {
            let (text, etag) = read_text(&file).await.unwrap();
            assert_eq!(text, "áé Linux\r\n");
            assert!(etag.is_some());
            std::thread::sleep(std::time::Duration::from_millis(5));
            std::fs::write(&path, "contenido cambiado fuera del editor").unwrap();
            let result = file
                .replace_contents_future(
                    b"no sobrescribir".to_vec(),
                    etag.as_deref(),
                    false,
                    gio::FileCreateFlags::NONE,
                )
                .await;
            assert!(result.is_err());
            assert_eq!(
                std::fs::read_to_string(&path).unwrap(),
                "contenido cambiado fuera del editor"
            );
            std::fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();
            assert!(read_text(&file).await.is_err());
            std::fs::write(&path, vec![b'a'; EDIT_LIMIT + 1]).unwrap();
            assert!(read_text(&file).await.unwrap_err().contains("8 MiB"));
        });
        std::fs::remove_dir_all(dir).unwrap();
    }
}
