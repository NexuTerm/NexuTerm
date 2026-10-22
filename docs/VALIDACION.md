# Validación de NexuTerm

## Menús e identidad gráfica — 22 de octubre de 2026

Entorno de esta revisión: Ubuntu 26.04.1, Rust 1.92.0, GTK 4.22.4,
libadwaita 1.9.1 y VTE 0.84.0. Rust y Xvfb se prepararon temporalmente
en `/tmp`, sin modificar el perfil del usuario ni instalar paquetes globales.

- `cargo build --locked`: correcto.
- `cargo test --locked`: 14 pruebas correctas; la integración SFTP se mantiene
  ignorada por defecto porque requiere el laboratorio remoto propio.
- `cargo fmt --all -- --check`: correcto.
- `cargo clippy --locked --all-targets -- -D warnings`: correcto.
- `python3 lab/check_ui.py menus`: correcto; dos ciclos de preferencias,
  acerca de, nueva conexión, búsqueda y gestor de sesiones. Comprueba que cada
  diálogo se abre, que el desplegable se cierra y que GTK encuentra el logo integrado.
- `python3 lab/check_ui.py menus --compact --scale 2`: correcto a 1024×768
  lógicos y escala 200 %; captura revisada.
- `python3 lab/check_ui.py sessions`: correcto, incluidos crear, editar,
  duplicar, eliminar y comprobar la persistencia con configuración temporal.
- `python3 lab/check_ui.py splits`: correcto; tres terminales, zoom y lateral.
- `python3 lab/check_ui.py close-splits`: correcto; cierre mediante la **×**
  de divisiones anidadas, reducción de tres paneles a uno y cierre de la rama
  original conservando la nueva. Comprueba cancelación de la sesión cerrada,
  permanencia de la pestaña y desaparición del botón al quedar una sola terminal.
- Recurso gráfico compilado, logo instalado idéntico al original y lanzador
  validado con `desktop-file-validate`. Instalación comprobada con `DESTDIR`
  temporal y binario de desarrollo; no se realizó una instalación global.

La prueba de argumentos RDP/VNC simula únicamente la búsqueda del ejecutable;
conserva las aserciones de puertos y seguridad, y comprueba el error si falta el cliente.
No equivale a conectar con un servidor RDP/VNC. Las pruebas gráficas usan Xvfb;
no validan Wayland nativo ni todos los portales de archivos. Se observan avisos
del entorno sobre EGL y portales, sin fallos en los escenarios comprobados.

## Mantenimiento de sesiones y navegador SSH integrado

Comprobado el 21 de octubre de 2026, con el mismo entorno WSL de abajo:

- `cargo build --locked`, `cargo fmt --all -- --check` y Clippy con `-D warnings`: correctos.
- `cargo test --locked`: 13 pruebas correctas; la integración remota está ignorada por defecto y se ejecuta expresamente con el comando siguiente.
- `cargo test --locked --test shared_sftp -- --ignored`: integración real correcta con el contenedor rootless propio, publicado solo en loopback. Comprueba conexión multiplexada, listado, paginación de 200 entradas, subida/descarga con contenido idéntico, nombres Unicode/con espacios/comillas/dólar, rechazo de conflictos locales y remotos, y cancelación antes de escribir.
- Prueba GTK del gestor: creación y edición mediante sus formularios, duplicado con ID distinto, borrado y lectura del resultado persistido. Conserva grupo, favoritos y las sesiones originales. Configuración ficticia en un directorio temporal.
- Navegador GTK autenticado contra el laboratorio SSH: captura revisada con el listado real de `/home/tester`, separado de los perfiles del usuario. Ninguna conexión a los equipos de las capturas del usuario.

Reproducir (con el laboratorio ya iniciado y `lab/lab.py trust` ejecutado):

```bash
cd ~/source/NexuTerm
~/.cargo/bin/cargo build --locked
~/.cargo/bin/cargo test --locked --test shared_sftp -- --ignored
python3 lab/check_ui.py sessions
python3 lab/check_ui.py ssh
python3 lab/check_ui.py ssh --compact --scale 2
python3 lab/check_ui.py splits
```

El script aísla configuración y D-Bus, comprueba el código de salida y los
marcadores de cada escenario, y guarda capturas/logs en `artifacts/`. El
escenario SSH permite exclusivamente el contenedor propio en 127.0.0.1:22222,
con la clave de laboratorio y la identidad de servidor copiada directamente
del contenedor. No se modifica la configuración SSH del usuario.

Estas pruebas no validan Wayland nativo, servidores del usuario ni la selección
manual de archivos a través de todos los portales de escritorio. No se ha probado
la cancelación a mitad de una escritura ni todos los cortes posibles de red.
Sigue pendiente la edición remota desde este nuevo panel, transferir árboles,
resolver conflictos con reemplazo y el resto de la ampliación profesional.

## Validación anterior: interfaz GNOME 0.2.0

El ejecutable `target/debug/nexuterm` se ha recompilado con la nueva ventana
libadwaita. Se han separado las acciones y preferencias en `src/actions.rs`.

| Comprobación | Resultado real |
|---|---|
| `cargo build --locked` | Correcto |
| `cargo test --locked` | 13 pruebas correctas: 12 de biblioteca y 1 del editor |
| `cargo fmt --all -- --check` | Correcto |
| `cargo clippy --locked --all-targets -- -D warnings` | Correcto |
| Arranque y acciones en Xvfb | Tres terminales mediante divisiones anidadas, zoom y lateral; marcador de éxito y tres salidas PTY con código 0 |
| Captura 1280×820, escala 100 %, tema oscuro | Revisada; cabecera, iconos, pestaña, perfiles ficticios y tres terminales |
| Captura 1024×768 lógicos, escala 200 % | Generada con framebuffer de 2048×1536; revisada por separado |

Los perfiles de la captura usan `demo.example.invalid`; la prueba no conecta
a ellos ni utiliza los destinos del usuario. Los artefactos están en
`artifacts/`, excluido por `.gitignore`.

Para reproducir las divisiones, añade `NEXUTERM_SMOKE_ACTIONS=1` al comando
de arranque aislado de abajo. `NEXUTERM_SMOKE_COMPACT=1` selecciona 1024×768
lógicos y `GDK_SCALE=2` activa el escalado al 200 %.

Estas pruebas verifican la interfaz y PTY locales, no la aceptación de la ampliación
completa. Faltan la ejecución simultánea, SFTP compartido, túneles, métricas,
llavero e integración de RDP/VNC en la interfaz, además del laboratorio remoto
y la validación Wayland/Ubuntu 24.04/Debian 13. El arranque con D-Bus aislado
produce avisos de portales del entorno, pero la app completa la prueba.

Si ya hay una instancia antigua abierta, ciérrala antes de ejecutar el binario
nuevo: GTK reutiliza la instancia registrada con el mismo identificador.

## Validación anterior: 0.1.0

Fecha: 21 de octubre de 2026.

## Entorno utilizado

- Ubuntu 26.04.1 LTS en WSL, x86_64.
- Rust/Cargo 1.99.1, instalados a nivel de usuario con rustup y sin modificar el PATH.
- GTK 4.22.4; VTE GTK4 0.84.0.
- Disponibles OpenSSH, `xfreerdp3`, TigerVNC `vncviewer`, GVfs, Xvfb y D-Bus.

## Resultados

| Comprobación | Resultado real |
|---|---|
| `cargo build --locked` | Compila el ejecutable de desarrollo |
| `cargo test --locked` | 8 pruebas: 7 de modelo/configuración/comandos y 1 de editor/GIO |
| `cargo clippy --locked --all-targets -- -D warnings` | Sin advertencias |
| `cargo fmt --all -- --check` | Correcto |
| Arranque gráfico Xvfb + D-Bus aislado | Proceso de app finaliza con código 0 |
| PTY VTE | Shell de prueba imprime marcador y su proceso termina con código 0 |
| Lectura de configuración | Carga perfiles ficticios SSH, RDP y VNC sin modificarlos |
| Captura de ventana GTK | Se generó y revisó visualmente; se ajustaron anchos y nombres |
| Instalación MobaXterm | SHA-256 de los seis originales coincide con el inventario inicial |

Las pruebas del modelo cubren host inválido/opciones, IPv6, argumentos sin shell,
ausencia de contraseñas en el esquema y en argumentos, formato de puertos RDP/VNC,
conservación de validación de certificados y persistencia con permisos 0600.

La prueba del editor/GIO utiliza archivos temporales locales: comprueba UTF-8 y CRLF,
rechazo de binarios/no UTF-8 y de archivos mayores de 8 MiB, y rechazo de una escritura
con etag obsoleto después de una modificación externa.
Eso valida el flujo local de GIO, no la implementación de etag de todos los servidores SFTP.

## Reproducir una prueba breve de arranque

```bash
cd ~/source/NexuTerm
cargo build --locked
test_home="$(mktemp -d /tmp/nexuterm-smoke.XXXXXX)"
mkdir -p "$test_home/config"
env HOME="$test_home" XDG_CONFIG_HOME="$test_home/config" \
    NEXUTERM_SMOKE_TEST=1 GSK_RENDERER=cairo \
    xvfb-run -a dbus-run-session -- ./target/debug/nexuterm
```

El modo de prueba ejecuta un comando fijo y local en `/bin/sh`, espera tres segundos
y sale. No realiza conexiones remotas. El marcador esperado en stderr es
`NEXUTERM_SMOKE_CHILD_EXIT=0`.
Puede añadirse `NEXUTERM_SCREENSHOT=/ruta/absoluta/captura.png` para guardar una captura;
esta variable debe apuntar a un archivo de prueba, porque la captura reemplaza el destino.
El modo normal arranca la shell del usuario; no usa ese comando fijo.

En la sesión aislada de WSL aparecieron avisos de portales, PipeWire/RealtimeKit y
montaje FUSE de GVfs. No impidieron el arranque ni la prueba PTY. SFTP real permanece
pendiente; la app usa GIO directamente y la prueba de arranque no verifica su montaje.

## Pendiente de aceptación

- Login SSH por contraseña/clave/agente y salto contra servidores de laboratorio.
- Navegación, autenticación, transferencias, cancelación y edición SFTP reales.
- Conexiones y desconexiones RDP/VNC, teclas, portapapeles y resize reales.
- Cierre de sesiones interactivas y comportamiento de los clientes frente a fallos de red.
- Pruebas en Ubuntu 24.04, Debian 13 y otras distribuciones; validación Wayland nativa.
- Empaquetado redistribuible para runtimes antiguos; Flatpak/AppImage aún no preparados.

No se ha inicializado Git, hecho commit, publicado ni desplegado el proyecto.
El repositorio HMI no se modificó. Los artefactos de compilación quedan en `target/`,
excluido por `.gitignore`.
