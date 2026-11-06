# Validación de NexuTerm

Este documento resume los resultados registrados para 1.0.0 y los laboratorios
reproducibles. Los resultados corresponden al estado y entorno de su ejecución;
no acreditan que las mismas pruebas se hayan repetido en el checkout actual.
Consulta el [README](../README.md) para el uso y los límites actuales y la
[documentación técnica](analysis/README.md) para la organización del código.

## Release 1.0.0 — 4 de noviembre de 2026

- Etiqueta: `v1.0.0`. Versión sincronizada en
  Cargo.toml/Cargo.lock; Acerca de utiliza `CARGO_PKG_VERSION`.
- `cargo test --locked --jobs 1 -- --test-threads=1`: 138 aprobadas y ocho omitidas
  por requisitos de entorno. `python3 -m unittest discover -s tests -p
  test_release.py`: cinco pruebas de empaquetado aprobadas.
- `python3 lab/check_markdown.py` y `python3 lab/check_markdown_media.py`:
  aprobados en claro/oscuro; selector nativo, fidelidad de fuente, deshacer,
  tablas, imágenes y cancelación HTTP. GTK/X11 virtual, no Wayland.
- `cargo build --locked --release --jobs 2`: correcto con Rust 1.93.1 en Ubuntu
  26.04.1 LTS; GTK 4.22.4, libadwaita 1.9.1, VTE 0.84.0.
- `python3 packaging/release.py deb --binary target/release/nexuterm --output-dir
  dist/1.0.0`: paquete amd64 1.0.0 generado. `dpkg-shlibdeps` advirtió de las
  diversiones del cargador de libc6 en el sistema constructor. Se revisó Depends,
  se comprobó el enlace con `packaging/release.py check` y no faltan bibliotecas.
- Paquete extraído sin instalarlo en el equipo: ELF idéntico al binario release y
  arranque X11 durante cuatro segundos con HOME/configuración/datos temporales.
  Esto no acredita instalación APT, actualización ni compatibilidad en otra distribución.
- SHA-256 del paquete en `SHA256SUMS`; hashes sin firma. Enlaces locales de los
  documentos revisados y `git diff --check` correctos.

Las pruebas previas sobre Ubuntu 24.04/Debian 13 corresponden a entregas anteriores;
no se han repetido para 1.0.0. Tampoco se ha completado la aceptación humana de
[ACEPTACION.md](ACEPTACION.md), la interacción gráfica de la papelera ni las
cuentas reales de todos los proveedores. Estas limitaciones se publican en las
[notas de la versión](RELEASE_1.0.0.md).

## Laboratorios reproducibles

El [README](../README.md#validación) enumera los comandos. Los escenarios y sus
requisitos están en `lab/` y las pruebas de lógica en `tests/` y los módulos Rust.
Los scripts gráficos requieren Xvfb y una sesión D-Bus aislada; los escenarios de
red deben usar los destinos temporales del laboratorio. No se deben ejecutar
contra equipos de producción.

- Markdown: `lab/check_markdown.py` y `lab/check_markdown_media.py` verifican
  fuente, formato, deshacer, tablas, selector de archivos e imágenes. La carga
  HTTP se comprueba contra loopback, con límites y cancelación.
- Documentos y SSH: `lab/check_recovery.py` y `lab/check_remote_editor.py`
  comprueban borradores, conflictos, guardado y sesiones autenticadas de prueba.
- Terminales e interfaz: `lab/check_ui.py` ofrece escenarios de pestañas,
  divisiones, sesiones, menús y conexiones; consulta sus argumentos antes de usarlo.
- Distribuciones: `lab/check_distributions.py` construye una instantánea en
  contenedores Ubuntu 24.04 y Debian 13. El procedimiento se explica en
  [ACEPTACION.md](ACEPTACION.md).

Los resultados dependen de la instantánea, las bibliotecas y los servidores
utilizados. No se mantienen aquí inventarios de perfiles personales, instalaciones
locales ni narraciones de cambios de interfaz sustituidos. Los resultados de
1.0.0 anteriores se conservan como registro fechado, no como prueba de revisiones
posteriores. Una revisión exclusivamente documental no vuelve a ejecutar esos
laboratorios ni acredita nuevas funciones.
