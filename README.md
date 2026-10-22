# NexuTerm

Aplicación Rust para Linux con terminal por pestañas y conexiones SSH, SFTP, RDP y VNC.
Primera versión funcional en desarrollo: no es una réplica completa de MobaXterm.

## Primera versión

- Interfaz GNOME con libadwaita: cabecera, barra de iconos con nombres accesibles, menú de acciones y pestañas reordenables.
- Filtro de conexiones y lateral ocultable con F9. Preferencias persistentes de tema, fuente, scrollback y visibilidad inicial del lateral.
- Divisiones horizontales y verticales anidadas, búsqueda literal en terminal, zoom, reconexión y desconexión desde el menú o la barra contextual.
- Terminal local mediante GTK4/VTE: PTY, Unicode, redimensionado y scrollback de 10.000 líneas.
- Ventana **Administrar sesiones** (Ctrl+Mayús+S): crear, editar, duplicar, eliminar con confirmación, buscar, agrupar y marcar favoritos.
- SSH mediante OpenSSH con usuario, puerto, clave y salto; contraseñas solicitadas en el PTY.
- Navegador remoto automático a la izquierda de cada pestaña SSH: ruta editable, carpeta de inicio, carpeta superior, páginas de 200 entradas, subida/descarga de archivos, progreso y cancelación. Usa SFTP sobre la conexión OpenSSH compartida del terminal, con la misma autenticación, puerto, clave y salto.
- Editor de texto mediante GIO, UTF-8 hasta 8 MiB, con comprobación etag antes de guardar y aviso de cambios sin guardar. El panel SFTP nuevo transfiere archivos; todavía no abre su editor remoto integrado.
- RDP con FreeRDP y VNC con TigerVNC: **ventanas propias**, lanzadas desde una pestaña que muestra diagnóstico y acepta los prompts del cliente.
- Configuración XDG versionada, escritura atómica y archivo privado; no contiene campos de contraseña.

El archivo de configuración está en `$XDG_CONFIG_HOME/nexuterm/sessions.json` o `~/.config/nexuterm/sessions.json`.
Si no se puede cargar o contiene una versión desconocida, se conserva y se deshabilita el guardado de sesiones.

## Compilar y ejecutar

Requisitos: Linux, Rust 1.92 o posterior, GTK >= 4.10, libadwaita >= 1.4 y VTE para GTK4 >= 0.70.
El objetivo nativo inicial es Ubuntu 24.04+ y Debian 13+; se ha comprobado el entorno indicado en `docs/VALIDACION.md`.
Para distros antiguas se necesitará un runtime empaquetado o backports compatibles: aún no se ha preparado Flatpak/AppImage.
El código no depende de apt durante la ejecución, pero no se afirma compatibilidad probada con todas las distribuciones.

Debian/Ubuntu (paquetes de desarrollo para compilar):

```bash
sudo apt update
sudo apt install build-essential pkg-config libglib2.0-dev-bin libgtk-4-dev libvte-2.91-gtk4-dev \
    libadwaita-1-dev libvncserver-dev libsecret-1-dev libssl-dev \
    openssh-client gvfs-backends dbus-user-session freerdp3-x11 tigervnc-viewer
```

En versiones que distribuyen FreeRDP 2, el paquete puede llamarse `freerdp2-x11`; NexuTerm busca `xfreerdp3` y `xfreerdp`.
VNC busca `vncviewer` o `xtigervncviewer`. GTK/VTE y sus paquetes de desarrollo tienen nombres distintos en otras distribuciones.

```bash
cd ~/source/NexuTerm
~/.cargo/bin/cargo build --locked
~/.cargo/bin/cargo run --locked
```

Rust se instaló a nivel de usuario sin modificar el PATH del shell. Si `cargo` ya está en tu PATH, puedes usarlo directamente.
Para otras máquinas, instala Rust desde [rust-lang.org](https://rust-lang.org/tools/install/).

### Instalación en el escritorio Linux

```bash
make build
sudo make install
```

Instala el binario, el lanzador del menú de aplicaciones y el logo de `logo/NexuTerm.png`
en `/usr/local`. El logo también queda integrado en el ejecutable y en **Acerca de NexuTerm**.
Para una instalación sin privilegios usa `make install PREFIX="$HOME/.local"`
(incluye `~/.local/bin` en tu PATH). Para preparar un paquete, usa `DESTDIR`.
La instalación registra el lanzador en el menú; no copia accesos directos al escritorio personal.

## Uso

1. Pulsa el icono de servidor (**Nueva conexión**, Ctrl+N) y selecciona SSH, RDP o VNC; ajusta host, puerto y usuario.
2. Guarda y pulsa **Conectar**. SSH abre una terminal; RDP/VNC abren el visor correspondiente y conservan una pestaña de diagnóstico.
3. Al abrir SSH aparece automáticamente **Archivos remotos**. Completa la autenticación y la comprobación de identidad en la terminal; el panel reutiliza esa conexión sin pedir otra contraseña. Doble clic abre una carpeta; también puedes escribir una ruta y pulsar Enter.
4. El icono **Subir archivo** permite elegir un archivo local. Para descargar, selecciona un archivo remoto y pulsa **Descargar**, o haz doble clic. Elige un nombre de destino que no exista: esta entrega no sobrescribe archivos existentes.
5. Copiar/pegar en terminal: **Ctrl+Shift+C / Ctrl+Shift+V**. El terminal interpreta los bytes pegados como cualquier shell interactiva.
6. Abre **☰ → Administrar sesiones** o pulsa **Ctrl+Mayús+S** para mantener los perfiles. El icono de lista de la cabecera abre la misma ventana. Usa el menú principal para dividir la terminal activa, buscar (Ctrl+Shift+F), ajustar el zoom y abrir preferencias. Ctrl+Shift+T crea una terminal local; Ctrl+O abre el editor.

El navegador utiliza **SFTP sobre SSH**, como el panel SSH de MobaXterm; no invoca el protocolo SCP clásico. Terminal y archivos comparten un socket de control propio y privado. Si no hay conexión autenticada, el navegador espera o permite reintentar; no inicia una conexión independiente ni modifica `~/.ssh/config`.

El lateral de conexiones se oculta al abrir una sesión SSH para dejar espacio al navegador remoto; F9 permite volver a mostrarlo. Cada pestaña SSH conserva su propio navegador y ruta. La navegación manual no sigue automáticamente los cambios de carpeta del shell.

## Límites conocidos

- La ampliación profesional sigue incompleta: ejecución simultánea, gestor de túneles, panel de métricas, llavero e integración RDP/VNC todavía no están conectados a la interfaz. Los módulos de backend en desarrollo no equivalen a funciones aceptadas.
- No hay todavía incrustación de los escritorios RDP/VNC dentro de GTK, ni pruebas reales de login contra servidores remotos.
- La UI cubre conexiones básicas; no expone todas las opciones de gateway, multimonitor, impresión, smartcard, codecs o TLS de los clientes externos.
- No se han implementado macros, diff, túneles configurables, Mosh, serie, servidores auxiliares, importación de perfiles MobaXterm ni todas sus herramientas.
- El explorador transfiere un archivo cada vez, no árboles de carpetas. No sigue enlaces simbólicos automáticamente. No sobrescribe archivos existentes y un error/cancelación puede dejar un destino parcial. Cancelar detiene nuevas solicitudes y espera la operación ya enviada; no revierte los bytes escritos. Se usan bloques de 64 KiB y listados de hasta 200 entradas por página.
- El editor solo acepta UTF-8; no abre binarios ni archivos mayores de 8 MiB. No guarda cuando el backend no ofrece etag. No hay resaltado de sintaxis todavía.
- No hay almacén propio de secretos; se utilizan los diálogos y agentes de los backends. No se debilita la validación de certificados ni de host keys.
- Cerrar una terminal envía SIGHUP al grupo del proceso PTY y destruye el terminal; procesos deliberadamente desacoplados no quedan bajo supervisión de la app.
- El binario compilado en una distribución reciente no garantiza compatibilidad con glibc más antigua. Construir en la base más antigua soportada antes de distribuir.

## Validación

```bash
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
```

Pruebas de biblioteca sin la interfaz GTK (el backend VNC requiere libvncclient):

```bash
cargo test --locked --no-default-features
```

Para una prueba breve de arranque con configuración aislada, consulta `docs/VALIDACION.md`.
Las pruebas de escritorio remoto necesitan servidores de laboratorio; no se usan equipos de producción.

## Análisis previo y fuentes

El inventario REA se encuentra en `docs/analysis/`. Incluye un informe, catálogo por ejecutable y propuesta de fases.
Especifica qué se observó estáticamente y qué sigue sin validación dinámica.
El código de NexuTerm se ha escrito como implementación propia sobre APIs públicas, sin copiar el pseudocódigo recuperado.

- [GTK Rust](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/)
- [VTE Rust](https://docs.rs/vte4/0.10.0/vte4/)
- [GIO](https://docs.gtk.org/gio/)
- [OpenSSH](https://www.openssh.com/manual.html)
- [FreeRDP](https://www.freerdp.com/)
- [TigerVNC](https://tigervnc.org/)
