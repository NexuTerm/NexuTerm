# Editor Markdown de NexuTerm

Los archivos `.md` y `.markdown` se abren en modo Visual en el editor integrado.
Código muestra toda la sintaxis. Ambas vistas utilizan el mismo TextBuffer, sin
conversión a HTML: guardar, recuperar borradores y comparar versiones conservan
la fuente Markdown, incluidas las marcas ocultas y las construcciones no interpretadas.
Las preferencias de apertura externa y la edición de otros tipos siguen vigentes.

## Formato y cinta

- Inicio: portapapeles, negrita, cursiva, tachado, código, listas, tareas,
  citas, títulos, selección y deshacer/rehacer.
- Insertar: referencias de enlaces e imágenes, tablas y bloques de código.
- Vista: zoom. El selector Visual permite alternar con Código.

La interfaz es nativa Rust/GTK4. El parser CommonMark/GFM `pulldown-cmark`
proporciona rangos de origen convertidos de bytes UTF-8 a posiciones GTK.
El formato aplica etiquetas sobre el buffer; las acciones de edición modifican
su fuente. Los documentos no ejecutan HTML ni scripts.

La vista visual está limitada a 256 KiB y 65.536 rangos de formato. El parseo se
realiza fuera del hilo GTK, con un trabajo por documento. Los archivos mayores
siguen como texto, dentro del límite general de 8 MiB del editor.

## Tablas e imágenes nativas

Los widgets se superponen sobre rangos del TextBuffer; no añaden caracteres de
objeto ni un segundo documento. Editar una celda actualiza inmediatamente su
rango Markdown, incluso antes del siguiente ciclo GTK; guardar y recuperar
siguen usando el mismo buffer. Se escapan las barras verticales nuevas. Los
cambios de estructura conservan los contenidos y alineaciones GFM y se deshacen
como una acción. La normalización de espacios solo afecta a una tabla cuya
estructura se modifica expresamente. Las celdas muestran su sintaxis Markdown;
la cinta admite negrita, cursiva, tachado y código en la selección de una celda.
Los estilos de párrafo e inserciones de bloques se aplican fuera de las celdas.

Límites por documento: 16 bloques visuales y 256 celdas; cada tabla admite hasta
32 filas y 12 columnas. Una celda admite 1024 caracteres y 4096 bytes. Las tablas
que exceden estos límites siguen como texto; no se truncan. Las imágenes en línea
junto a otro texto conservan su referencia literal. Código permite editar toda
la sintaxis y cualquier construcción no cubierta por los widgets.

En **Insertar → Imagen**, **Explorar…** abre el selector nativo, con filtros de
imágenes y todos los archivos. Seleccionar rellena la referencia, sin modificar el
documento hasta pulsar Insertar: ruta relativa dentro de la carpeta del documento
y URI local fuera de ella o para documentos SSH. No copia ni sube el archivo.

Las imágenes locales se resuelven respecto al documento. La carga automática
queda dentro de su carpeta, comprobando también los enlaces simbólicos; fuera
de ella requiere Cargar. HTTP/HTTPS y las rutas SSH siempre requieren pulsar
Cargar. SSH utiliza el destino autenticado que ya tiene vinculado el documento,
no la ruta temporal local ni una conexión nueva. Se rechazan credenciales en URL,
SVG, HTML y otros esquemas. El original nunca se modifica. `curl` es necesario
para HTTP, con redirecciones limitadas a HTTP/HTTPS y límites de tamaño y tiempo.

El límite es 4 MiB y cuatro millones de píxeles por imagen, con vistas previas
escaladas, dos trabajos simultáneos y una caché de ocho texturas por documento.
Cerrar la vista cancela las solicitudes HTTP/SSH; las lecturas y decodificaciones
locales en curso terminan de forma acotada fuera del hilo GTK. La autorización de
carga externa no se persiste ni se reutiliza tras cerrar el documento.

`python3 lab/check_markdown_media.py` ejecuta GTK en claro/oscuro y un servidor
HTTP exclusivamente de loopback. Comprueba fuente y estado modificado intactos,
edición/estructura/deshacer, formato de celda, imagen local, carga HTTP expresa,
404, límites con/sin Content-Length, redirección prohibida y cancelación al cerrar.
`python3 lab/check_remote_editor.py` incluye lectura binaria de imagen por SFTP,
rutas con espacios y rechazo de imágenes de más de 4 MiB.
