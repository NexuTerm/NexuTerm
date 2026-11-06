# NexuTerm 1.0.0

Primera release pública, 4 de noviembre de 2026. Etiqueta: `v1.0.0`.

## Incluido

Terminales con pestañas y divisiones, perfiles y conexiones SSH/SFTP, FTP, Telnet,
RDP y VNC, exploradores, cola de transferencias, edición local/remota con conflictos
y recuperación, comparación de versiones, espacios de trabajo, chats de agentes,
ejecución supervisada, recetas, intervenciones e informes. El editor Markdown
incluye cinta agrupada, tablas editables, imágenes y selección nativa de archivos.
El [CHANGELOG](../CHANGELOG.md) resume la entrega; el [README](../README.md)
explica los controles, permisos y límites de cada función.

## Descarga e instalación

Descarga desde [GitHub Releases](https://github.com/NexuTerm/NexuTerm/releases/tag/v1.0.0).
GitHub incluye los archivos de código fuente asociados a la etiqueta.

La publicación contiene **solo código fuente**, sin `.deb` ni binarios adjuntos.
Para instalar un paquete Debian debes compilarlo y generarlo localmente con las
instrucciones siguientes. Las dependencias se calculan desde el binario y el
sistema de construcción; no se garantiza compatibilidad con una distribución
más antigua que aquella donde se genere.

## Compilar desde la etiqueta

Rust >= 1.92, GTK >= 4.10, libadwaita >= 1.4 y VTE GTK4 >= 0.70. Consulta en el
README los paquetes de desarrollo, LibVNCClient y TLS necesarios.

```bash
git clone https://github.com/NexuTerm/NexuTerm.git
cd NexuTerm
git checkout v1.0.0
cargo build --locked --release
./target/release/nexuterm
# Opcional, con Python >= 3.11, dpkg-dev y binutils:
make package
dpkg-deb --field dist/nexuterm_1.0.0_amd64.deb Version Architecture Depends
sudo apt install ./dist/nexuterm_1.0.0_amd64.deb
```

## Datos y actualización

Es la primera release pública; la configuración y los borradores de las versiones
de desarrollo conservan sus comprobaciones de formato y versión. El programa
rechaza formatos desconocidos en lugar de convertirlos silenciosamente. Antes de
reemplazar una instalación de desarrollo, cierra la aplicación y guarda una copia
de tus directorios XDG de configuración y datos de NexuTerm. No publiques esas
copias: pueden contener historial de chats, comandos o documentos recuperados.
No se promete volver a un binario anterior conservando datos escritos después.

## Límites conocidos y aceptación

- RDP/VNC no ofrecen toda la configuración de gateways, multimonitor, audio,
  impresión o portapapeles. La compatibilidad con todos los servidores no está certificada.
- FTP y Telnet transmiten sin cifrar. SSH conserva la validación de host keys;
  los permisos de las herramientas de IA deben elegirse según el entorno.
- Guardar por SFTP sustituye el inodo; no garantiza ACL/atributos extendidos ni
  bloqueo frente a otros clientes. Los conflictos deben revisarse antes de guardar.
- Markdown visual admite hasta 256 KiB. Las tablas y las imágenes tienen los
  límites documentados; SVG no tiene vista previa. Código permite consultar
  y editar la fuente completa.
- Macros, Mosh, serie, servidores auxiliares y Flatpak/AppImage no están incluidos.
- Las pruebas automatizadas no sustituyen la aceptación humana, las cuentas
  reales de proveedores ni las pruebas Wayland. La eliminación mediante papelera
  se ha compilado; su interacción gráfica todavía requiere aceptación manual.

Verificación de esta entrega: 138 pruebas Rust, cinco pruebas de empaquetado y
los laboratorios Markdown/medios en claro y oscuro aprobados; compilación release,
paquete, dependencias y arranque aislado del binario extraído comprobados.

El [registro de validación](VALIDACION.md) separa los resultados de esta versión
de los históricos. [ACEPTACION.md](ACEPTACION.md) contiene la comprobación operativa
con operadores y cuentas autorizadas; esa aceptación no se presupone realizada.
