# Editor Markdown visual — implementación inicial y siguientes pasos

Rama: `feat/markdown_viewer`. Referencia inspeccionada: pluMd 0.2.5 en
`/home/mjulia/source/pluMd`. Se ha implementado una primera versión nativa: parser CommonMark/GFM con rangos,
vistas Visual/Código, cinta Inicio/Insertar/Vista, formato, listas, deshacer/rehacer,
zoom y formularios para referencias. No incluye aún widgets de celdas o imágenes,
ni equivalencia completa con PluMd. El orden siguiente conserva el diseño y los
criterios pendientes para ampliar esta base.

## Hallazgos de la revisión

PluMd utiliza Electron y CodeMirror 6. `livePreview.js` oculta delimitadores sobre
el árbol sintáctico y añade decoraciones y widgets; no convierte todo el documento
a HTML para después volver a serializarlo. `commands.js` aplica transacciones sobre
la fuente para formato, listas y selección. `ribbon.js` organiza Inicio, Insertar,
Vista y Ayuda. `tables.js` y `tablaModelo.mjs` separan la edición visual de tablas
de su representación Markdown. `images.js` resuelve recursos relativos al documento;
las imágenes HTTP requieren carga expresa en la vista visual.

La idea reutilizable es conservar Markdown como fuente única y representar su
formato encima. El código de referencia tiene licencia MIT: si se incorpora código,
deben conservarse el aviso de copyright y la licencia. No se ha copiado código.

NexuTerm usa Rust/GTK4 y un TextBuffer por documento en `src/files.rs`. El guardado
y `recovery_ui::snapshot` incluyen caracteres ocultos. Hay límites UTF-8, conflictos
por etag, borradores privados y guardado remoto protegido. Estas rutas deben seguir
siendo las únicas responsables de persistir documentos. WebKitGTK 6 no está
disponible mediante pkg-config en el entorno inspeccionado; incorporar la interfaz
web de PluMd exigiría un nuevo motor, dependencias y puente de edición/guardado.

## Comportamiento objetivo

- Abrir `.md` y `.markdown` en modo visual cuando se use el editor integrado.
  Mantener las preferencias de apertura externa y el editor de otros tipos.
- Documento editable con tipografía proporcional, ancho de lectura cómodo y temas
  claro/oscuro. Alternancia Visual/Código sin conversión destructiva ni cambio del
  estado de modificación por cambiar de vista.
- Cinta compacta: Inicio (estilos, negrita, cursiva, tachado, listas y tareas),
  Insertar (enlaces, imágenes, tablas, citas y código), Vista (código y zoom).
- Deshacer/rehacer agrupado por acción, atajos y estado de formato de la selección.
- Tablas con edición de celdas; imágenes locales relativas con límites de tamaño;
  recursos remotos solo mediante una acción expresa. Ningún HTML ni script del
  documento debe ejecutarse. No descargar recursos al abrir un Markdown SSH.
- Conservar YAML, HTML literal, extensiones no reconocidas, Unicode, saltos de línea
  y delimitadores que no se hayan editado. No imponer anchos de columna no estándar
  ni presentar colores/fuentes arbitrarios de Word como Markdown portable.

## Orden de trabajo y aceptación

1. **Base de documento:** detectar extensiones sin distinguir mayúsculas; incorporar
   un parser CommonMark/GFM mantenido con rangos de origen, evitando un parser
   artesanal basado en expresiones regulares. Convertir correctamente offsets UTF-8
   a posiciones del TextBuffer. Probar código cercado, sintaxis anidada, escapes,
   documentos incompletos y límites antes de ocultar delimitadores.
2. **Visual/Código:** aplicar formato y ocultación sobre el mismo buffer. Preservar
   cursor, selección, deshacer, estado modificado y borrador. No acumular callbacks
   ni reparsar ilimitadamente el documento en cada pulsación. Probar edición junto
   a marcas ocultas, selección con teclado, IME, pegado y búsqueda.
3. **Cinta de edición:** comandos de formato y bloques con operaciones de deshacer
   coherentes. Conservar foco/selección al pulsar botones. No añadir otro explorador,
   gestor de pestañas, sistema de plugins o formato de persistencia.
4. **Tablas e imágenes:** widgets vinculados a rangos del documento, modificaciones
   locales de la fuente y rutas ligadas a cada documento, no a un directorio global.
   Conservar código/Markdown no compatible mediante la vista Código.
5. **Integración y regresión:** aperturas locales/SSH, comparar versiones, guardar,
   recuperar después de SIGKILL y reabrir espacios. Abrir/cambiar de modo/guardar
   sin editar debe conservar el contenido. Verificar conflictos y archivos ausentes.
6. **Aceptación visual:** capturas claro/oscuro y ventanas estrechas, uso real de
   botones, selección y atajos; documentos grandes, caracteres combinados y emoji.
   Compilaciones y pruebas sobre las distribuciones objetivo del proyecto.

El núcleo nativo evita introducir Electron y un segundo entorno de ejecución. Se añade pulldown-cmark 0.13.6 sin características opcionales (MSRV declarado 1.71.1),
compatible con el mínimo Rust 1.92 del proyecto. No se ejecuta HTML del documento.
El parseo está limitado a 256 KiB y 65536 rangos, fuera del hilo GTK, con un trabajo
por documento; los demás archivos siguen siendo editables como texto.
