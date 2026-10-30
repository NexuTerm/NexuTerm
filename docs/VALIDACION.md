# Validación de NexuTerm

Este documento conserva resultados históricos. Cada apartado corresponde al estado
y entorno de su ejecución; no acredita que las mismas pruebas se hayan repetido en
el checkout actual. Las implementaciones sustituidas se mantienen como historial.
Consulta el [README](../README.md) para el uso y los límites actuales y la
[documentación técnica](analysis/README.md) para la organización del código.

## Ejecución supervisada — 30 de octubre de 2026

- `./recompilar.sh -d` y `./recompilar.sh -r`: compilaciones completadas con
  Rust 1.93.1. `cargo test --quiet`: 112 pruebas aprobadas y cinco ignoradas de
  entorno; `cargo test --quiet --no-default-features --lib`: 62 aprobadas.
- Lógica de ejecución: directorio explícito, Unicode, stdout/stderr separados,
  código de salida, cancelación del grupo local, timeout, límite conjunto de
  4 MiB y drenaje de salida adicional. Sin estado de terminación confirmado,
  el resultado es desconocido; no se inventa un código de salida.
- Historial: permisos 0700/0600, escrituras atómicas, exportación binaria exacta
  sin sobrescritura, captura máxima de 4 MiB, 100 registros desde cuatro hilos,
  cuota, archivos corruptos y enlaces simbólicos. La falta de espacio reservado
  impide iniciar el comando. Las capturas periódicas solo se escriben si cambia
  la salida; la evidencia final se espera antes de considerar terminado el trabajo.
- `python3 lab/check_supervised.py`: GTK en oscuro y claro; rechazar no ejecuta,
  aprobar ejecuta una vez, directorio con comillas, salida y retorno reales,
  timeout, cancelación, exclusión del historial y fallo de reserva. Incluye una
  petición MCP por el socket real, caducidad antes de aprobar y desconexión del
  solicitante. Los resultados cuya persistencia falla requieren retirada explícita
  de memoria; el historial conservado no se elimina por esa retirada.
- El mismo laboratorio mata realmente la aplicación con SIGKILL después de una
  captura periódica: al reiniciar conserva la salida parcial, marca el resultado
  desconocido y no repite el comando. El cierre normal con una ejecución activa
  pide confirmación, cancela el grupo y espera el registro final antes de salir.
- `python3 lab/check_remote_editor.py`: integración con sshd aislado aprobada,
  incluidos comandos supervisados sobre el ControlMaster autenticado, directorio
  con comillas, ambos flujos, retorno 13 y resultado desconocido ante timeout,
  cancelación o retorno SSH 255. La conexión compartida permanece utilizable.
  También pasan las regresiones gráficas de edición y recuperación remota.
- `python3 lab/check_recovery.py`: ocho casos aprobados, incluidos SIGKILL,
  conflicto, original ausente, recuperación remota sin conexión y cierre bloqueado
  si falla el autoguardado.
- Se ajustaron los fixtures ACP y Responses al mensaje de sistema que precede al
  primer mensaje del usuario, conservando las comprobaciones de autorización.

Límites: pruebas locales y SSH de laboratorio; no acreditan todos los servidores
ni una sesión gráfica Wayland. Cancelar el cliente SSH no demuestra que el proceso
remoto haya terminado: se muestra resultado desconocido y se exige comprobarlo
antes de repetir. La ejecución no es un sandbox y no se reanuda automáticamente.

## Resolución de versiones del editor remoto — 30 de octubre de 2026

- `./recompilar.sh -d` y `./recompilar.sh -r`: compilaciones completadas.
  `cargo test`: 100 pruebas aprobadas y cinco de entorno ignoradas en la suite
  predeterminada. `cargo test --no-default-features --lib`: 53 aprobadas.
- `lab/check_remote_editor.py`: servidor sshd desechable en loopback, claves y
  known_hosts privados, sin modificar configuración global ni usar producción.
  Prueba Rust de integración aprobada: lectura, guardados repetidos y sustitución
  de inodo, conservación de permisos 0640 y bit especial 2640, rechazo de enlaces,
  binarios y exceso de tamaño, destino incorrecto y eliminación de temporales.
- Ampliada esa prueba con revisión Base/Local/Original: una comparación obsoleta
  no se acepta; aceptar no escribe; los cambios posteriores de permisos también
  producen conflicto; la base avanza tras guardar. La recuperación para revisión
  conserva el guardado bloqueado hasta aceptar explícitamente el original.
- GTK/Xvfb en claro y oscuro: tres versiones exactas y diferencias resaltadas,
  resultado editable, confirmación obligatoria, cancelación sin perder el borrador,
  rechazo del original que cambió después de comparar y del resultado editado
  durante la comprobación. Aplicar conserva el original intacto y persiste texto y
  base; Guardar publica después y mantiene los permisos. Capturas revisadas:
  `artifacts/editor-compare-dark.png` y `editor-compare-light.png`.
- Recuperación SSH desde un JSON anterior sin `base_text`: se indica base ausente,
  se conserva el texto, se revisa y aplica expresamente sin escribir el remoto.
  Regresión de apertura SSH desde espacios de trabajo aprobada sin duplicación.
- `lab/check_recovery.py`: ocho escenarios aprobados, incluyendo SIGKILL real,
  restauración/guardado con etag, conflicto, original ausente, perfil remoto
  eliminado, descarte confirmado, cierre normal y fallo de captura que impide
  cerrar y conserva el editor modificado.
- Pruebas unitarias nuevas: reconstrucción exacta de 81 parejas de texto con
  Unicode, CRLF, inserciones, borrados y ausencia de salto final; contraste grande
  acotado y optimización de prefijo común; compatibilidad de borradores anteriores,
  persistencia de la base y rechazo de una base que no coincide con la huella SSH.

Límites: la resolución es manual y no afirma fusionar automáticamente los cambios.
El detalle se acota a dos millones de celdas y 200.000 líneas por versión; por
encima se muestran regiones completas con aviso, sin truncar los textos. La base
privada requiere hasta 8 MiB adicionales por borrador. SFTP no ofrece aquí ACL ni
xattrs y la comprobación antes de rename no bloquea a otros clientes.
No se han hecho commits ni push.

## Recientes por agente y cola persistente — 30 de octubre de 2026

- `./recompilar.sh -d` y `./recompilar.sh -r`: compilaciones completadas.
- `cargo test`: 96 pruebas aprobadas; cinco pruebas de entorno permanecen
  ignoradas en la suite predeterminada. `cargo test --no-default-features --lib`:
  49 pruebas aprobadas. `git diff --check`: sin incidencias.
- `python3 lab/check_chat_recents.py`: GTK/Xvfb, dos perfiles API de prueba y un
  fixture nativo Codex, sin cuentas reales. Verifica vista inicial completa,
  editor visible, respuesta en curso que termina sin cancelarse al abrir recientes,
  escritura que abre una conversación sin contexto anterior,
  sustitución dentro de la misma pestaña, selección y continuación, listas por
  agente, reinicio del proceso y paginación nativa. El protocolo confirma una
  creación nativa y dos reanudaciones; seleccionar no reenvía turnos anteriores.
  Comprueba también permisos privados y captura de los mensajes nativos cargados.
  Capturas: `artifacts/chat-recents-initial.png` y `chat-recents-reload.png`.
- `python3 lab/check_chat_state.py`: aprobados los seis escenarios de recuperación
  API, ACP, fallos/reintento/desconexión ACP, estado Codex y 75 turnos con archivo
  completo. Las copias de espacios continúan desconectadas hasta reanudación.
- `lab/check_workspace_views.py`, `lab/check_workspace_visuals.py` y
  `lab/check_workspace_transfers.py`: aprobados cursor/selección/desplazamiento,
  estado del explorador, foco/zoom de terminal, lectura del chat sin salto forzado
  y restauración FTP/SFTP desconectada. La prueba de lectura espera la posición
  real asignada por GTK; no toma la posición pendiente del snapshot como prueba
  de que la vista ya esté lista. Una ejecución concurrente bajo compilación
  también agotó el tiempo fijo de la prueba del editor; su repetición aislada
  pasó sin cambiar el código del editor.
- `lab/check_file_browser.py ftp --queue --colon-user` y la variante
  SFTP `--queue`: aprobados selección múltiple, portapapeles y arrastre en ambos
  sentidos, carpetas anidadas/vacías, nombres Unicode y porcentaje literal,
  rechazo de sobrescritura, reintento manual, pausa, diario privado y recuperación
  tras SIGKILL con inicio explícito. FTP probado con usuario que contiene dos
  puntos. Se aprobó también la regresión FTP Windows CE.
- `tests/shared_sftp.rs`: dos pruebas reales de laboratorio aprobadas (SSH
  compartido y SFTP independiente), incluyendo cancelación después de transferir
  bytes, ausencia de publicación parcial y reintento. El contenedor propio quedó
  detenido después de las comprobaciones.

Límites: no acredita cuentas reales de GitHub/Codex ni servidores FTP de producción.
Las subidas FTP no garantizan exclusión frente a otro cliente y pueden dejar un
archivo parcial tras interrupción. Los JSONL antiguos sin identidad de agente y
contexto de continuación siguen consultables en Historial local completo; no se
atribuyen automáticamente a un perfil por su nombre visible.

## Historial completo y continuidad de chats; script de compilación — 28 de octubre de 2026

- Historial incremental privado JSONL, separado de las 128 tarjetas de pantalla.
  Consulta de conversaciones cerradas por páginas de 25 mensajes y exportación
  completa a archivo nuevo. Captura por defecto, exclusión explícita y borrado
  confirmado sin resurrección. Límite global de 1 GiB/1000 conversaciones, entrada
  de 2 MiB y cola de 64 elementos/4 MiB por chat; la importación inicial está
  acotada a 16 MiB. La lectura del agente se pausa ante disco lento y la captura
  se detiene con aviso ante errores/cuota, conservando los registros existentes.
- Contexto API de intercambios completados ligado al destino del perfil. Se
  rehidrata en memoria, sin peticiones HTTP ni repetición de herramientas; el
  siguiente mensaje explícito conserva el contexto. ACP negocia `loadSession`
  o `sessionCapabilities.resume`, no crea otra sesión y rechaza permisos durante
  la reproducción. El envío espera confirmación. Los fallos de carga y pérdida
  del proceso permiten reintentar conservando mensajes, borrador y adjuntos.
- Los espacios y copias automáticas conservan adjuntos pendientes y el nivel de
  razonamiento anunciado por el modelo. Las rutas ausentes no se leen ni recrean.
  Los formatos anteriores siguen siendo consultables: no se inventan referencias
  ni mensajes que no guardaron.
- `./recompilar.sh -d` y `./recompilar.sh -r`: compilaciones reales correctas,
  también al invocar el script desde otro directorio. Opciones excluyentes,
  `Cargo.lock` respetado, dos trabajos por defecto, reutilización del toolchain
  instalado y errores propagados. `bash -n`, ayuda y casos de argumentos inválidos,
  Cargo inexistente y fallo de Cargo comprobados con sus códigos de salida.
- Se encontró un fallo de renombrado de metadatos de Rust en el directorio de
  artefactos release anterior. Una salida limpia en el mismo volumen funcionó;
  se conservó el directorio antiguo como respaldo y se regeneró `target/release`.
  No se modificó la configuración global ni se cerró la instancia gráfica con
  trabajo abierto. La prueba GTK usa los hooks de debug; el ejecutable release
  se acredita por compilación, no por marcadores que solo existen en debug.
- `cargo test --locked --jobs 2`: **80 pruebas aprobadas**, cinco omitidas por
  defecto (cuatro de laboratorio y una que requiere una pantalla). Incluye
  límites y corrupción de historial, exportación sin sobrescritura, exclusión y
  borrado, contrapresión ante disco bloqueado, contexto API y soporte/carga ACP.
  El protocolo API de ambos formatos conserva un turno entre clientes nuevos y
  no realiza HTTP al restaurar. ACP cubre carga, reanudación sin replay, rechazo
  por falta de soporte y errores, sin nueva sesión ni permisos aceptados.
- `python3 lab/check_chat_state.py`: seis casos GTK con proveedores sintéticos:
  API, ACP, fallo y reintento, desconexión y reintento, razonamiento/adjuntos y
  75 turnos. Se verifican 128 tarjetas visibles y 153 mensajes archivados, consulta
  y paginación del historial local, y ausencia de lecturas/recreación de adjuntos.
- Límites: las API mantienen 256 KiB/128 entradas por solicitud; se conserva hasta
  2 MiB/129 entradas del último contexto completado para no perder su respuesta.
  No es contexto ilimitado. El cliente nativo conserva el límite de 128 elementos
  por turno con interrupción y aviso. Agentes ACP sin capacidad de carga y copias
  antiguas sin referencia no pueden reanudarse. Sin proveedores reales, servicios
  de producción, commits ni push de esta entrega.

## Pie de chat compacto — 27 de octubre de 2026

- Dos filas como máximo: cuenta, cuota y acceso; contexto, tokens y avisos.
  Sin salto de línea de las etiquetas, con detalle completo seleccionable y en
  ayuda emergente. Se conservan inicio/cierre de sesión, acceso web y avisos de
  errores; se elimina el aviso de éxito «Conversación recuperada».
- Compilación de depuración correcta. Pruebas gráficas aprobadas: cuenta/cuota
  con comprobación de las dos filas, conversación Codex, autenticación con y sin
  cuenta existente, cierre de sesión, chats independientes, API Responses y Chat
  Completions. Cuatro escenarios de espacios con chats aprobados, incluida
  reanudación correcta sin aviso y fallo protegido.
- Inspección visual de la captura de Codex en oscuro: cuenta/cuota y cierre de
  sesión en la primera fila y contexto en la segunda, sin líneas adicionales.

## Recuperación automática de sesión — 27 de octubre de 2026

- Copia independiente por ventana cada dos segundos, cuando no hay otra captura
  o escritura/transferencia en curso. Conserva borradores antes de publicar la
  estructura compatible, selección, rutas y mensajes visibles/borradores de chat.
  No modifica espacios guardados con nombre ni requiere guardarlos manualmente.
- Archivo privado 0600 y directorio 0700, reemplazo atómico, sincronización y
  bloqueo auxiliar estable ante el reemplazo. Las ventanas activas se excluyen
  del listado; se conserva el checkpoint tras cerrar normalmente. El cierre
  fuerza una última escritura y, ante error, permite cancelar o cerrar sin
  actualizar la copia. Los formatos dañados o desconocidos no se sobrescriben.
- Recuperación al arrancar o desde el menú. Añade estructuras desconectadas,
  sin ejecutar terminales, agentes o turnos. La copia anterior solo se retira
  después del reemplazo duradero; un reintento no duplica lo restaurado. El
  descarte pide confirmación y no borra documentos ni borradores. Se limitan las
  copias pendientes a 32 sin expulsar automáticamente datos anteriores.
- `cargo build --locked --jobs 2`: ejecutable de depuración compilado.
  `cargo test --locked --jobs 2`: **73 pruebas aprobadas**, cuatro integraciones
  de laboratorio omitidas por defecto. Tres pruebas nuevas cubren permisos,
  publicación y bloqueo, separación de ventanas, corrupción/enlaces simbólicos,
  retirada sin resurrección y capacidad sin eliminación de copias anteriores.
- `python3 lab/check_session_recovery.py`: seis procesos GTK aislados y cuatro
  grupos de comprobaciones aprobados. Incluyen dos capturas periódicas sucesivas
  sin guardar espacio, SIGKILL real, oferta al arrancar, documento y chat con la
  última edición, divisiones y selección, ausencia de conexiones nuevas, fallo
  de almacenamiento durante el relevo y reintento sin duplicados, sustitución
  duradera antes de retirar la copia anterior y descarte confirmado. También
  cierre antes del primer intervalo, cancelación de cierre ante fallo, recuperación
  tras corregir el almacén y preservación de un formato dañado.
- Regresiones aprobadas: cuatro escenarios de chats de espacios, siete de
  documentos asociados a espacios, ocho de borradores y cuatro de espacios de
  terminales; recorrido de menús a 1280 px. Los scripts usan Xvfb/X11, D-Bus y
  HOME/XDG temporales, sin servicios reales ni cambios de originales.
- Revisión visual del documento y chat recuperados. Se corrigió la vista vacía
  del chat que aún aparecía sobre mensajes recuperados sin cliente conectado,
  y se oculta el historial del proveedor hasta conectar; la prueba automática
  comprueba que la vista vacía quede oculta. Formato de módulos nuevos, sintaxis
  Python y `git diff --check` correctos. Sin dependencias nuevas, commit ni push.
- Límites: persiste la estructura compatible y la copia visible de chats, no
  historial ilimitado, API/ACP reanudables, adjuntos, foco/zoom/scrollback ni pestañas
  FTP/SFTP/RDP/VNC. Una caída puede perder las ediciones posteriores a la última
  captura completada. No se acreditan reanudaciones con una cuenta real ni se ha
  generado un nuevo paquete de distribución.

## Chats asociados a espacios — 27 de octubre de 2026

- Integración en la paleta independiente de IA: identidad, agente referenciado,
  modelo, carpeta, mensajes visibles, borrador, orden, selección y visibilidad.
  Los chats vivos se reutilizan sin sustituir los cambios actuales. Las copias
  se guardan en el JSON privado y atómico del espacio, con límites explícitos.
- Restauración sin iniciar agentes, consultar credenciales ni enviar mensajes.
  Los perfiles eliminados conservan la copia consultable. La capacidad de ocho
  chats se comprueba antes de restaurar cualquier pestaña del espacio.
- Codex publica la identidad de conversación creada/reanudada. La acción de
  reanudación es explícita, arranca en solo lectura y bloquea el envío hasta
  confirmación. Se conserva también el borrador editado después de restaurar.
  Un error mantiene los mensajes y el borrador y bloquea incluso la invocación
  directa del botón Enviar; no crea una conversación nueva ni un turno.
- `cargo build --locked --jobs 2`: compilación correcta del ejecutable de depuración.
  `cargo test --locked --jobs 2`: **70 pruebas aprobadas**, cuatro pruebas de
  laboratorio omitidas por defecto; ninguna fallida. Se añadió cobertura de
  esquema, compatibilidad con espacios sin chats, duplicados, límites y
  conservación del archivo anterior ante datos inválidos. El protocolo nativo
  verifica la identidad publicada al crear una conversación.
- `python3 lab/check_workspace_chats.py`: cuatro escenarios aprobados:
  guardar, restaurar, reanudar Codex y fallo al reanudar. El guardado incluye una
  segunda ejecución que lee el archivo producido por la primera. Comprueba
  contenido Unicode, bloques de código, mensajes de herramientas, copia de
  respuesta, borradores, modelos, referencias de perfiles eliminados, identidad,
  selección, reutilización, capacidad sin restauración parcial y cierre del
  último chat. El protocolo simulado confirma `thread/resume` al destino y
  carpeta guardados, sin `thread/start` ni `turn/start`. Sin modelos reales ni
  servicios externos; Xvfb/X11 y XDG temporal.
- Regresiones gráficas aprobadas: siete escenarios de documentos de espacios y
  cuatro de espacios de terminales. También Codex con captura real de mensajes
  y referencia nativa, nuevo chat con agente/modelo y cierre independiente,
  dos turnos API Responses y Chat Completions, catálogo ACP, copia exacta del
  Markdown al portapapeles e inicio de sesión Codex con cuenta simulada existente.
- Inspección visual de la captura de reanudación: conversación recuperada,
  pestañas independientes y borrador conservado. `git diff --check` y compilación
  Python de los scripts nuevos/modificados correctos. Sin dependencias nuevas,
  commit, push ni paquete de distribución nuevo. Clippy sigue sin estar disponible
  en el toolchain local.
- Alcance: copia de hasta los 128 mensajes visibles y el borrador al guardar el
  espacio. No acredita historial ilimitado, reanudación API/ACP, adjuntos pendientes,
  autoguardado ante caída ni validación contra una cuenta real de proveedor.

## Documentos y borradores asociados a espacios — 27 de octubre de 2026

- El espacio conserva la identidad de cada editor, su origen y la referencia al
  borrador actual; no duplica el texto. Antes de guardar se conservan los
  borradores abiertos modificados. Un fallo en esa captura evita sustituir el
  espacio anterior. La escritura se realiza en segundo plano y el diálogo impide
  guardar, abrir o eliminar simultáneamente.
- La restauración respeta orden y selección para los documentos recuperados;
  los editores ya abiertos conservan posición y cambios. Los documentos locales
  limpios leen el original actual; los borradores SSH se recuperan sin red y los
  documentos SSH limpios esperan una apertura expresa sobre una sesión existente.
- `cargo build --locked --offline --jobs 2`: correcto, sin advertencias.
  `cargo test --locked --offline --jobs 2`: 69 pruebas correctas; cuatro
  integraciones ignoradas por defecto. Los casos nuevos validan referencias,
  identidad frente a cambios de versión/destino, UUID, duplicados y rechazo de
  documentos dentro de divisiones de terminal; los datos inválidos no sustituyen
  el archivo anterior.
- `python3 lab/check_workspace_documents.py`: siete escenarios correctos con
  configuración y datos ficticios aislados. Incluye guardado seguido de SIGKILL,
  reapertura con orden/selección, reapertura sin duplicar editores ni perder una
  edición posterior, borrador ausente, original eliminado, registro remoto dañado,
  registro local dañado y fallo de almacenamiento. Abrir y cerrar el original
  del registro local dañado conserva intacto ese registro.
- `python3 lab/check_remote_editor.py`: integración SSH real y cinco recorridos
  GTK correctos. El caso nuevo abre un documento SSH limpio desde el espacio,
  requiere una acción explícita, lee el original actual sin modificarlo, conserva
  su posición y no crea otra sesión ni duplica el editor al reabrir el espacio.
  Esta prueba no restaura una terminal SSH del espacio: la conexión de laboratorio
  ya existe antes de abrir el documento.
- `python3 lab/check_workspaces.py`: cuatro escenarios previos correctos.
  `python3 lab/check_recovery.py`: ocho escenarios previos correctos.
  Captura de la reapertura de documentos revisada visualmente.
- Se detectó un problema del laboratorio: el primer recorrido de redimensionado
  del explorador heredó Wayland aunque se había arrancado Xvfb. La consulta del
  display confirmó `GdkWaylandDisplay`; fijando `GDK_BACKEND=x11` pasaron las mismas
  aserciones. Los scripts de esta entrega que requieren Xvfb ahora fijan X11 en
  el proceso de prueba; no se cambia el backend de la aplicación normal. Se
  repitieron los recorridos de documentos, recuperación y SSH en ese entorno.
- `python3 lab/check_local_explorer.py`: tres recorridos correctos con X11.
  `python3 lab/check_ui.py menus`: correcto.
  `python3 -m unittest discover -s tests -p test_release.py`: cinco correctas.
  Formato de los módulos de espacios/recuperación y `git diff --check`: correctos.
  Clippy sigue sin estar instalado. La acción Guardar rechaza versiones ausentes
  también si se invoca directamente, sin depender solo del estado del botón.
- Continúan pendientes el historial de chats, las pestañas FTP/SFTP y de
  escritorio remoto, el resto de estado visual y la recuperación automática de
  estructura. No se han hecho commits ni push ni generado un paquete de release.

## Vinculación de borradores SSH y guardado atómico — 27 de octubre de 2026

- Los borradores nuevos conservan la huella SHA-256 del contenido original.
  `Vincular a SSH` exige una conexión existente del mismo perfil y parámetros
  configurados de destino, compara esa huella y no escribe en el original.
  `Revalidar SSH` utiliza la huella actualizada tras el guardado. Los borradores
  anteriores sin huella siguen recuperándose para comparación y copia local.
- El guardado del editor remoto deja de utilizar SCP: SFTP publica un temporal
  verificado mediante POSIX rename. Se conservan y comprueban propietario,
  grupo y permisos; falta de soporte o errores de metadatos bloquean la escritura
  directa. Se retiró el requisito de `scp` del comprobador de ejecución.
- `cargo build --locked --offline --jobs 2`: correcto, sin advertencias.
  `cargo test --locked --offline --jobs 2`: 67 pruebas correctas; cuatro
  integraciones ignoradas por defecto.
- `python3 lab/check_remote_editor.py`: integración real y cuatro recorridos GTK
  correctos. Comprueba reemplazo del inodo, permisos, conflicto, destino distinto,
  borrador antiguo sin huella, directorio sin permiso de creación, retirada de
  temporales y conservación de la copia local. Se interrumpe SSH después de
  observar una subida parcial de 8 MiB en el temporal: el destino publicado
  permanece completo, sin contenido parcial.
- Los recorridos GTK cubren edición normal en oscuro/claro, recuperación con
  vinculación y guardado, revalidación y persistencia de la nueva huella, y
  rechazo gráfico de un original modificado externamente. Captura de la
  recuperación vinculada revisada visualmente.
- `python3 lab/check_recovery.py`: ocho escenarios anteriores correctos, incluidos
  SIGKILL, original ausente, descarte y bloqueo del cierre ante fallo de captura.
  `python3 lab/check_workspaces.py`: cuatro escenarios correctos.
  `python3 -m unittest discover -s tests -p test_release.py`: cinco correctas.
- Formato de los módulos de recuperación, backend remoto y su prueba de
  integración y `git diff --check`: correctos. Clippy no está instalado en el
  toolchain. No se generó un paquete de release ni se hicieron commits o push.
- Límites: el reemplazo cambia el inodo y no garantiza ACL/atributos extendidos;
  no bloquea a escritores externos entre la última comprobación y el renombrado.
  La interrupción probada ocurre durante la subida, no durante un fallo de energía
  o la confirmación del rename. No se ha probado un servidor que omita la extensión
  POSIX rename; la ausencia se comprueba antes de crear el temporal.

## Recuperación de documentos — 26 de octubre de 2026

- `cargo build --locked --offline --jobs 2`: correcto, sin advertencias.
  `cargo test --locked --offline --jobs 2`: 67 pruebas correctas y cuatro
  integraciones ignoradas por defecto. Los casos nuevos cubren permisos,
  persistencia, versiones y límites, borradores dañados, enlaces, invalidación
  de escrituras obsoletas y exportación sin reemplazar archivos existentes.
- `python3 lab/check_recovery.py`: ocho escenarios correctos con Xvfb y D-Bus,
  HOME/XDG temporales y texto ficticio. Incluye SIGKILL del proceso GTK después
  de la captura periódica, recuperación en otro proceso, guardado con etag y
  retirada del borrador, conflicto con comparación visible, original ausente,
  borrador remoto sin conexión, descarte confirmado, conservación de la última
  edición antes de cerrar y bloqueo del cierre si el destino de recuperación
  no puede utilizarse. La exportación se valida mediante el backend real; el
  selector nativo de archivos no se automatiza en esta prueba.
- El escenario GTK remoto utiliza un borrador sintético sin servidor ni perfil.
  `python3 lab/check_remote_editor.py` añade la integración SSH/SCP real: verifica
  que el origen remoto y el texto del borrador sobreviven a la eliminación de su
  copia temporal. También pasan los recorridos gráficos oscuro/claro de edición
  y guardado remoto. No se ha implementado la subida de borradores recuperados.
- `python3 lab/check_local_explorer.py`: correctos los tres recorridos de editor
  local y apertura externa. `python3 lab/check_workspaces.py`: cuatro escenarios
  correctos. `python3 lab/check_ui.py menus`: correcto, ahora con datos XDG
  aislados para no consultar borradores personales durante la prueba.
- `python3 -m unittest discover -s tests -p test_release.py`: cinco correctas.
  Formato de `recovery.rs` y `recovery_ui.rs` y `git diff --check`: correctos.
  Captura del editor recuperado revisada visualmente. Clippy sigue sin estar
  disponible en el toolchain de esta máquina; no se generó un paquete de release.
- Los cambios de espacios de trabajo previos permanecen en el árbol de trabajo.
  No se han hecho nuevos commits ni push durante esta entrega. Se conservaron
  ajustes simultáneos ajenos a la tarea en `src/style.css`.

## Primera entrega de espacios de trabajo — 26 de octubre de 2026

- Antes de implementar se publicó `a443439` en `origin/main`: corrección del
  desplazamiento que cubría el botón de copiar. Compilación, suite Rust,
  las dos pruebas gráficas de copia (Codex/ACP) y cinco pruebas de empaquetado
  correctas; `HEAD...origin/main` quedó en `0 0` antes de los cambios nuevos.
- Toolchain de esta ejecución: Rust 1.93.1 instalado por REA, GTK 4.22.4,
  libadwaita 1.9.1 y VTE 0.84.0. No se actualizaron dependencias ni lockfile.
- `cargo build --locked --offline --jobs 2`: correcto; binario de desarrollo
  reconstruido. `cargo test --locked --offline --jobs 2`: 63 pruebas correctas,
  cuatro integraciones ignoradas por defecto.
- `python3 lab/check_workspaces.py`: cuatro procesos GTK aislados correctos:
  guardar/renombrar, reapertura, carpeta ausente y perfil eliminado. Verificados
  notas, permisos privados, divisiones anidadas, conservación de pestañas,
  ausencia de ejecución automática y carpeta del proceso iniciado expresamente.
  Capturas de diálogo y disposición revisadas en `artifacts/workspaces-*.png`.
- `python3 lab/check_ui.py menus` y `close-splits`: correctos.
- `python3 lab/check_remote_editor.py`: integración SSH/SCP real correcta y
  recorridos GTK oscuro/claro correctos, con servidor y claves desechables.
  Es una regresión del editor y navegador, no una prueba de reconexión desde
  un espacio restaurado.
- `python3 -m unittest discover -s tests -p test_release.py`: cinco correctas.
  Formato de los módulos Rust nuevos y `git diff --check`: correctos.
- Xvfb se ejecutó desde la instalación local de laboratorio. Clippy no está
  instalado en este toolchain; su comprobación no pudo ejecutarse. No se generó
  un paquete de release ni se validaron otras distribuciones.
- Esta entrega no cierra la fase de recuperación completa: documentos pendientes,
  historial de chats y demás fases continúan en [MEJORAS.md](MEJORAS.md).

## Whisper local instalado — 23 de octubre de 2026

- Motor oficial whisper.cpp, etiqueta v1.9.5, commit
  `d1be6fde11ac6e0407606b4e42fe72d34add8037`, compilado para CPU e instalado
  en `.local/whisper/bin/whisper-cli`. Licencia y manifiesto en esa misma carpeta.
  CMake se preparó solo en `/tmp`; no se instalaron paquetes globales.
- Modelo multilingüe `ggml-tiny.bin`, 77.691.713 bytes, SHA256 comprobado contra
  los metadatos LFS del proveedor:
  `be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21`.
- Descubrimiento automático del motor local sin cambiar preferencias explícitas.
  Idioma inicial español; configuración accesible en los ajustes de agentes.
  La instalación opcional queda ignorada por Git y fuera de `target`.
- Audio español generado localmente con eSpeak, sin micrófono ni reproducción.
  La detección automática inicial confundió el idioma; con español explícito
  transcribió las dos primeras frases. Tiny no garantiza transcripción exacta.
- `python3 lab/check_dictation.py /tmp/nexuterm-whisper-es.wav`: correcto.
  Grabador sustituido por una copia del WAV; Whisper y su modelo son reales.
  Confirmada la inserción de la transcripción en el borrador GTK y el
  descubrimiento automático con HOME/configuración temporales.
- `cargo test --locked --offline --jobs 2`: 42 pruebas correctas; tres
  integraciones remotas anteriores ignoradas. Formato Rust, sintaxis Python y
  `git diff --check` correctos. No se probó la captura del micrófono real.


## Chat gráfico nativo de Codex — 23 de octubre de 2026

Sustituye las implementaciones de consola y login mediante PTY descritas abajo.
El servidor oficial `codex app-server` comunica mensajes JSON por tuberías
privadas. Chat GTK con respuestas en streaming, contexto entre turnos,
interrupción y diálogos de aprobación; modo inicial de solo lectura.
Los perfiles locales Codex existentes se reconocen automáticamente.
Autenticación por `account/read` y `account/login/start`, con reutilización
de sesión y apertura del navegador desde GTK. No se accede a auth.json.

Pruebas reproducibles nuevas en `tests/codex_chat.rs`: streaming en dos turnos
con una sola conversación, rechazo de aprobación, interrupción, login simulado
y salida malformada. El servidor ficticio no usa modelos ni credenciales.
La prueba gráfica en `lab/check_ai.py` comprueba el compositor y la respuesta.

Cabecera compacta y editor inferior siguiendo la referencia del usuario.
Historial real por carpeta, selector real de modelos y razonamiento, contexto
mediante rutas seleccionadas y dictado local configurable con PipeWire/Whisper.
Prueba de dictado con grabador y transcriptor ficticios: detención, inserción
del texto y eliminación del audio. No se grabó el micrófono real.

Verificación: `cargo test --locked --offline --jobs 2`: 41 pruebas correctas,
tres integraciones remotas anteriores ignoradas. `lab/check_ai.py`: doce
comprobaciones gráficas, incluida la aprobación con rechazo explícito.
Handshake, `account/read` y `thread/list` confirmados además con el Codex instalado
y HOME/CODEX_HOME vacíos, sin OAuth ni inferencia. Se revisaron las capturas
oscura y clara del chat y el diálogo de aprobación. En esta validación inicial aún faltaban whisper-cli y el modelo; la instalación
local posterior se documenta arriba.

Límites: sin paridad completa con VS Code. Las
solicitudes de herramientas no implementadas se rechazan. No se ha repetido
OAuth real ni ejecutado una inferencia contra la cuenta del usuario.


## Reutilización de sesión de Codex y aviso de Brave — 23 de octubre de 2026

- Diagnóstico local: el registro de Codex muestra callback y dos intercambios
  OAuth completados. El informe de Apport de las 15:30:08 corresponde al proceso
  principal de Brave (`/opt/brave.com/brave/brave`, SIGTRAP), seis segundos
  después del segundo intercambio. No se imprimieron URLs OAuth ni tokens.
- `codex login status` confirma una sesión de ChatGPT disponible. El estado de
  la sesión se consultó sin volver a ejecutar el login real ni cambiar credenciales.
- Corregido el relogin innecesario: el botón consulta primero al CLI y reutiliza
  la sesión de ChatGPT si existe. Solo inicia OAuth si no hay una sesión de ese
  tipo. Consulta acotada a cinco segundos y 8 KiB por canal, con cancelación y
  cierre supervisado; su salida no se muestra ni se guarda en logs de NexuTerm.
- `cargo test --locked --offline --jobs 2`: 36 pruebas correctas; tres
  integraciones remotas anteriores ignoradas. `lab/check_ai.py`: once
  comprobaciones correctas, incluida la regresión que impide reiniciar OAuth o
  abrir el navegador cuando el CLI confirma una sesión de ChatGPT.
- Formato Rust, sintaxis Python y `git diff --check`: correctos.
- Límite: no está confirmada la causa interna del SIGTRAP de Brave. No se
  modificó su configuración, no se eliminaron informes de fallo y no se cerraron
  ventanas del navegador del usuario para reproducirlo.


## Inicio de sesión web de Codex — 23 de octubre de 2026

- Botón **Iniciar sesión web en Codex** en la paleta. Ejecuta `codex login`
  mediante la misma PTY y el mismo cierre supervisado que los agentes locales.
- Se utiliza el perfil Codex seleccionado o uno existente; si no hay ninguno,
  se crea un perfil cuando el ejecutable está disponible y la configuración
  permite guardarlo. Los argumentos del chat no se pasan al login ni se alteran.
- Codex gestiona el navegador y sus credenciales; NexuTerm no lee ni copia
  `auth.json`, ni guarda tokens OAuth en la configuración de sesiones.
- Confirmados CLI instalado `codex-cli 0.160.1`, `codex login --help` y la
  documentación oficial de autenticación. No se ejecutó el login real.
- `python3 lab/check_ai.py`: diez comprobaciones correctas en Xvfb aislado.
  Los dos casos nuevos verifican perfiles existentes/nuevos y delegación en un
  navegador ficticio. `CODEX_HOME` y HOME temporales; sesión real intacta.
- `cargo test --locked --offline --jobs 2`: 36 pruebas correctas y tres
  integraciones remotas anteriores ignoradas. Formato y diff sin errores.
- Compilaciones debug y release correctas; `./nexuterm` apunta al ejecutable
  optimizado actualizado.
- Pendiente de validación manual: completar OAuth con una cuenta real en el
  navegador y abrir después una conversación autenticada en el CLI de Codex.


## Chat gráfico común por agente — 23 de octubre de 2026

- Todas las plantillas utilizan el mismo panel GTK; no se abre ninguna PTY de IA.
  Codex utiliza app-server; OpenCode, Claude Code, Gemini CLI y Copilot utilizan
  ACP (con adaptador externo para Claude). Responses y Chat Completions comparten
  compositor, selector de modelos y herramientas de terminal.
- Las plantillas crean otro perfil y lo seleccionan al guardar. Comprobación
  gráfica específica: Codex permanece en el combo y OpenCode inicia session/new
  en el mismo chat. Se validó también el cambio de perfil y el segundo modelo ACP.
- Configuración y autenticación privadas por perfil. MCP, skills y agentes se
  guardan por agente y se adaptan a sus formatos; las capacidades finales dependen
  del proveedor. El botón de acceso desaparece tras confirmar una cuenta Codex.
- Puente MCP/funciones para terminales existentes, con aprobación gráfica de
  lectura y ejecución. Prueba con dos procesos locales ficticios, uno identificado
  como sesión remota: selección, aprobación, envío y lectura correctos. No se
  ejecutó ningún comando en una máquina remota real.
- SSH permite seleccionar contraseña o clave, con explorador nativo del archivo.
  La contraseña la solicita OpenSSH y no se almacena en el perfil. Prueba unitaria
  de argumentos SSH y prueba gráfica del formulario correctas.
- Se elimina el borde interno de terminal; se conserva el marco exterior.
- `cargo test --locked --offline --jobs 2`: 46 pruebas correctas; tres integraciones
  remotas ignoradas. `lab/check_ai.py`: 14 comprobaciones correctas, más la nueva
  regresión de plantillas ejecutada por separado, también correcta.
- OpenCode instalado: arranque gráfico por ACP con catálogo de modelos gratuitos,
  inicialización MCP real, descubrimiento de skills, agente
  principal y subagente correctos. ACP anuncia modelos sin consumir inferencias.
- No se validaron cuentas ni inferencias de proveedores externos. Claude ACP,
  Copilot y Gemini CLI no están instalados en este entorno. Su instalación y los
  permisos de cuenta son necesarios para probar el acceso real. No se ha confirmado
  la causa del fallo previo del navegador al cerrarse.

## Validación anterior del primer transporte de IA — 23 de octubre de 2026

Registro histórico, sustituido funcionalmente por el chat gráfico común descrito arriba.

- Paleta lateral derecha independiente, accesible desde cabecera, menú y
  Ctrl+Mayús+I. CLI local, OpenCode, Responses y Chat Completions; plantilla Ollama.
- Perfiles persistentes con URL, modelo, instrucciones y límites. Claves API en
  memoria o Secret Service, vinculadas al perfil y servidor; no se serializan.
- OpenCode recibe MCP, rutas de skills, agentes y subagentes mediante
  `OPENCODE_CONFIG_CONTENT`; no se modifican sus archivos de configuración.
  Las plantillas MCP quedan desactivadas hasta configurarlas y habilitarlas.
- Catálogo gratuito actualizable desde OpenCode instalado o, si falta, desde
  el catálogo público. No se seleccionan modelos de pago como alternativa.
  Gratuidad y disponibilidad quedan sujetas a las condiciones del proveedor.
- `cargo build --locked --offline --jobs 2` y la variante `--release`: correctos.
  El lanzador `./nexuterm` apunta al ejecutable optimizado actualizado.
- `cargo test --locked --offline --jobs 2`: 36 pruebas correctas, tres
  integraciones remotas preexistentes ignoradas; cero doctests.
- `python3 lab/check_ai.py`: ocho comprobaciones correctas en Xvfb aislado:
  PTY interactiva; dos turnos Responses y Chat Completions; cierre de petición
  HTTP pendiente al salir; guardado de extensiones; claves en memoria y llavero
  efímero; y MCP/skills/agentes en OpenCode 1.17.15 instalado.
- El MCP ficticio completó la inicialización real; OpenCode reconoció una skill,
  un agente principal y un subagente. No se ejecutaron inferencias con ellos.
- Pruebas HTTP con curl real y servidores efímeros en loopback: contexto,
  formatos JSON, cancelación y errores sin exponer el cuerpo del proveedor.
- Capturas del chat y del diálogo de extensiones revisadas; páginas de ajustes
  desplazables para evitar desbordamientos de altura. Formato Rust de archivos
  añadidos/modificados y `git diff --check` correctos.
- Toolchain Rust 1.92 y Xvfb preparados únicamente en `/tmp`, sin instalar
  paquetes ni modificar configuración global. Las pruebas de red y GUI necesitan
  acceso a loopback, D-Bus y al servidor gráfico; el sandbox lo bloquea.
- Limitaciones: respuestas API completas, sin streaming ni ejecución de
  herramientas; MCP/skills/subagentes por perfil requieren OpenCode. Codex y
  Claude Code conservan su configuración nativa. No se validaron inferencias
  contra proveedores externos ni el acceso real a promociones gratuitas.


## Paletas de terminal clara y oscura — 23 de octubre de 2026

- Fondo gris carbón/texto gris claro en oscuro; marfil/texto gris oscuro
  en claro. Paletas ANSI, cursor y selección definidos explícitamente.
- Actualización inmediata de terminales existentes según el tema efectivo
  de libadwaita, incluido el tema del sistema. La suscripción se desconecta
  al destruir cada terminal.
- Prueba de contraste: texto normal >= 7:1 y texto seleccionado >= 4,5:1.
  Colores ANSI para texto >= 4,5:1 salvo el negro ANSI del tema oscuro,
  conservado para prompts con fondos de color.
- Comprobaciones GTK aisladas `lab/check_ui.py colors --theme dark` y
  `--theme light`: capturas revisadas y cambio en vivo comprobado.
- Los colores RGB definidos por el shell o programas no se sustituyen.


## Compatibilidad FTP Windows CE — 23 de octubre de 2026

- Diagnóstico de solo lectura en el servidor indicado: `SYST` devuelve
  Windows_CE 7.0 y `LIST /FFSdisk` contiene entradas con hora de 24 horas,
  sin AM/PM. GVfs las presenta como una carpeta vacía.
- Cuando GVfs devuelve cero entradas, se contrasta el LIST original. Un
  listado DOS/Windows CE reconocido utiliza el transporte compatible, sin
  abrir terminal y con credenciales solo por stdin privado. Otros formatos
  no reconocidos muestran un error en lugar de afirmar que la carpeta está vacía.
- `curl` añadido a las dependencias y comprobaciones del paquete.
- Pruebas del parser: nombres con espacios, tamaños, listados vacíos y
  rechazo de rutas peligrosas. 25 pruebas Rust y cinco de empaquetado correctas.
- `lab/check_file_browser.py ftp --windows-ce`: navegación, descarga,
  subida, rechazo de sobrescritura y cierre correctos con un servidor local
  que reproduce el formato observado. No se escribieron archivos en el
  equipo Windows CE del usuario.
- Este modo lista hasta 8 MiB y comprueba existencia antes de subir; FTP
  no permite garantizar exclusión frente a escrituras simultáneas ajenas.


## Explorador gráfico FTP y SFTP — 23 de octubre de 2026

Esta implementación sustituye los flujos FTP por consola y la autenticación
SFTP mediante terminal descritos en las entradas históricas siguientes.

- Dos paneles integrados, local/remoto, navegación por carpetas, subida,
  descarga, progreso, cancelación y cierre de pestaña.
- FTP utiliza GIO/GVfs; el paquete declara y comprueba `gvfs-backends`.
  Contraseñas mediante memoria o llavero, sin archivos netrc.
- SFTP conserva OpenSSH, clave/agente/salto y solicita contraseña/frase de
  paso con un diálogo gráfico. No abre terminal ni shell remoto.
- `cargo test --locked`: 23 pruebas correctas; las tres integraciones
  externas se omiten por defecto.
- `cargo test --locked --test shared_sftp -- --ignored`: dos pruebas
  correctas en el contenedor propio, incluidas transferencias y cancelación.
- `python3 -m unittest discover -s tests -p test_release.py -v`: cinco correctas.
- `lab/check_file_browser.py ftp --keyring` y `sftp` con clave y
  `--password`: correctas bajo Xvfb; navegación, UTF-8, subida/descarga
  verificadas byte a byte, sobrescritura rechazada y cierre.
- `lab/check_ui.py sessions`, `close-splits` y `lab/check_basic.py`:
  correctas. Formato, Clippy con `-D warnings` y diff sin errores.
- `make release`: ejecutable optimizado y `dist/nexuterm_0.2.0_amd64.deb`
  generados; dependencias del paquete y comprobación de ejecución correctas.
- El entorno Xvfb muestra avisos de portales/FUSE/EGL; GVfs también emite un
  aviso de metadatos al crear un destino FTP. Las transferencias comprobadas
  finalizaron correctamente. No se han probado todos los servidores reales.

## Credenciales FTP en la configuración — 23 de octubre de 2026

- FTP muestra usuario y contraseña en el formulario. La contraseña puede
  guardarse con libsecret/Secret Service o mantenerse solo durante la ejecución.
  El perfil JSON conserva únicamente usuario y referencia, nunca la contraseña.
- La referencia del llavero se vincula al servidor, puerto y usuario. Cambiar
  cualquiera exige contraseña nueva; editar sin cambiar destino conserva la
  anterior. Duplicar no copia el secreto y eliminar limpia su referencia.
- tnftp recibe `-N` con un archivo netrc privado 0600, escapado y sin contraseña
  en argumentos, entorno o URI. La pestaña y el cierre de la app eliminan el
  temporal explícitamente; las pruebas verifican ambos caminos. No se modifica
  el archivo `~/.netrc` personal. Usuarios FTP con correo o dominio admitidos.
- `cargo test --locked`: 22 pruebas correctas; tres integraciones ignoradas por
  defecto. Clippy con `--all-targets -- -D warnings`: correcto.
- `python3 lab/check_basic.py`: Telnet y FTP correctos. FTP se configura desde
  el formulario con contraseña ficticia con espacios, comillas y barras; el
  servidor verifica el acceso automático, sin nuevo prompt en la terminal.
- `python3 lab/check_basic.py --keyring`: correcto, con D-Bus, HOME, datos y
  Secret Service temporales. Se guarda, se edita conservando contraseña, se
  rechaza reutilizarla para otro destino y se recupera leyendo la referencia
  persistida después de vaciar la memoria. No se toca el llavero personal.
- `python3 lab/check_ui.py sessions` y cinco pruebas de empaquetado: correctos.
  El release declara libsecret detectado y GNOME Keyring o KeePassXC.

## Telnet, FTP y SFTP independiente — 23 de octubre de 2026

- Selector con seis protocolos y puertos/campos específicos. Persistencia de
  Telnet, FTP y SFTP comprobada; las contraseñas no forman parte del perfil.
- Telnet y FTP utilizan sus clientes dentro del PTY de NexuTerm. FTP emplea
  modo pasivo y espera a la asignación inicial de tamaño antes de arrancar;
  así su lector de login no se interrumpe por un redimensionado inicial.
- SFTP reutiliza OpenSSH y el navegador existente con `ssh -N`: no abre shell.
  Su autenticación aparece en una zona de la pestaña, que se oculta al recibir
  el listado. Un botón permite volver a mostrarla. Cerrar la pestaña detiene
  el cliente y el proceso SSH. La sesión SSH con lateral permanece disponible.
- `cargo test --locked`: 20 pruebas correctas, tres integraciones ignoradas
  por defecto. Clippy con `--all-targets -- -D warnings`: correcto.
- `cargo test --locked --test shared_sftp -- --ignored`: dos pruebas correctas
  contra el laboratorio local; navegación, subida, descarga y cancelación con
  SSH compartido y SFTP independiente.
- `python3 lab/check_basic.py`: clientes Telnet y FTP reales dentro de GTK
  contra servidores efímeros exclusivos de loopback. Telnet verifica entrada
  y salida; FTP verifica usuario/contraseña ficticios, PWD y QUIT. Esta prueba
  no cubre transferencias de datos FTP ni servidores de producción.
- `python3 lab/check_ui.py sessions` y `python3 lab/check_ui.py sftp`: correctos;
  campos/puertos de los seis protocolos y ocultación de autenticación SFTP.
- Cinco pruebas de empaquetado correctas. El `.deb` declara `inetutils-telnet`
  y `tnftp`, además de OpenSSH y las bibliotecas enlazadas.
- Telnet y FTP son protocolos sin cifrado; se indica en el formulario y README.

## Confirmación de certificados RDP — 23 de octubre de 2026

- Si falla la validación TLS, se obtiene el certificado en otra conexión sin
  autenticar. GTK muestra servidor, titular, emisor, validez y huella SHA-256.
  Cancelar es la opción predeterminada. Aceptar autoriza solo ese certificado
  en esa misma conexión TLS, sin guardar confianza ni desactivar PKI globalmente.
- `cargo test --locked`: 18 pruebas correctas; dos integraciones ignoradas por
  defecto. Clippy con `--all-targets -- -D warnings`: correcto.
- `cargo test --locked --test rdp_certificate -- --ignored`: correcto con el
  laboratorio local. Verifica que no hay sesión autenticada antes de aprobar,
  que la huella coincide con la CA copiada del laboratorio, aceptación/rechazo,
  cierre durante la espera, nueva confirmación al reconectar y PKI con CA válida.
- `python3 lab/check_ui.py rdp-certificate`: correcto. El diálogo se muestra y
  Cancelar tiene el foco inicial; pulsarlo rechaza la autorización. Captura
  revisada: `artifacts/rdp-certificate-dialog.png`.
- No se ha accedido al servidor de la captura del usuario ni se ha aceptado su
  certificado; su huella debe contrastarse por un canal independiente.

## RDP integrado — 23 de octubre de 2026

- RDP utiliza el motor IronRDP existente y el visor GTK compartido con VNC.
  No se ejecutan FreeRDP ni Remmina. La contraseña solo se utiliza en memoria.
- `cargo test --locked`: 18 pruebas correctas; una integración SFTP ignorada.
  Se comprueban liberaciones de botones, arrastre, rueda, teclas extendidas,
  Unicode y la ausencia de lanzamiento de visores externos para RDP.
- Clippy con `--all-targets -- -D warnings`, cinco pruebas de empaquetado,
  `bash -n lab/start.sh` y `git diff --check`: correctos.
- `python3 lab/check_ui.py rdp`: correcto frente al XRDP del contenedor local,
  con usuario ficticio, TLS y CA copiada directamente del laboratorio. Se recibe
  imagen 1280×800, se envían eventos de teclado y ratón y se verifica el cierre
  de la conexión y su trabajador al cerrar la pestaña. Captura revisada:
  `artifacts/rdp-desktop.png`, con el escritorio Fluxbox dentro de NexuTerm.
- El laboratorio selecciona Xvnc explícitamente; la imagen no instala Xorg.
- Regresiones `python3 lab/check_vnc.py` y `python3 lab/check_ui.py sessions`:
  correctas. No se han probado servidores Windows/NLA ni portapapeles o audio.
- El release incluye IronRDP compilado en el ejecutable; el `.deb` declara las
  bibliotecas TLS detectadas y `ca-certificates`, sin clientes RDP externos.
  Los certificados y nombres de servidor se verifican. Las CA privadas pueden
  indicarse mediante una ruta PEM en la pestaña RDP.

## VNC integrado y dependencias del release — 23 de octubre de 2026

- VNC se abre dentro de una pestaña GTK, mediante LibVNCClient, sin ejecutar
  Remmina ni TigerVNC. El trabajo de red queda fuera del hilo de interfaz.
- `cargo test --locked`: 16 pruebas correctas. Incluyen la prohibición de lanzar
  un visor externo para VNC y el cálculo de coordenadas con escala y márgenes.
  La integración SFTP sigue ignorada por defecto.
- `cargo clippy --locked --all-targets -- -D warnings`: correcto.
- `python3 -m unittest discover -s tests -p test_release.py`: cinco pruebas
  correctas, incluida la ausencia de dependencias de visores VNC externos.
- `python3 lab/check_vnc.py` y `python3 lab/check_vnc.py --password`: correctos.
  Servidor RFB 3.8 simulado, efímero y exclusivo de loopback, sin cuentas reales.
  Comprueban una imagen 64×48 con píxeles conocidos, traducción de coordenadas,
  mensajes de teclado/ratón/rueda y cierre efectivo del socket y del trabajador
  al cerrar la pestaña. La variante de contraseña ejercita el desafío VNC;
  el servidor simulado no verifica criptográficamente la respuesta.
- Captura `artifacts/vnc-integrated.png` revisada: escritorio visible dentro de
  NexuTerm y controles de contraseña, conexión, estado y desconexión.
- El framebuffer se limita a 16.777.216 píxeles (64 MiB RGBA). Se conserva solo
  la última imagen pendiente; no hay una cola creciente de fotogramas.
- La contraseña no se guarda en la configuración ni se pasa a procesos externos.
  El botón Conectar impide iniciar varios trabajadores simultáneos en la misma pestaña.

`make release` genera el `.deb`; `dpkg-shlibdeps` detecta las bibliotecas del
binario, incluida LibVNCClient, y se declaran OpenSSH, certificados y
sesión D-Bus. `make check-runtime` solo exige el cliente externo SSH y
las bibliotecas enlazadas. La instalación manual se detiene antes de copiar
archivos si faltan dependencias; `DESTDIR` permite el empaquetado.

Las pruebas usan Xvfb; no validan Wayland nativo ni todas las variantes de
servidor/autenticación VNC. No hay sincronización de portapapeles, cambio de
resolución remota ni configuración de certificados personalizados en la interfaz.
Una conexión en curso puede tardar hasta los timeouts del backend en terminar
si se cierra durante el establecimiento o una lectura bloqueada.
La instalación real con APT no se ha realizado: requiere autenticación de
administrador. El paquete no es autónomo sin conexión ni garantiza compatibilidad
con distribuciones más antiguas que la de construcción.

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

## Entorno utilizado en la validación 0.1.0

- Ubuntu 26.04.1 LTS en WSL, x86_64.
- Rust/Cargo 1.99.1, instalados a nivel de usuario con rustup y sin modificar el PATH.
- GTK 4.22.4; VTE GTK4 0.84.0.
- Disponibles OpenSSH, `xfreerdp3`, TigerVNC `vncviewer`, GVfs, Xvfb y D-Bus.

## Resultados de la validación 0.1.0

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

## Pendiente de aceptación registrado en 0.1.0

- Login SSH por contraseña/clave/agente y salto contra servidores de laboratorio.
- Navegación, autenticación, transferencias, cancelación y edición SFTP reales.
- Conexiones y desconexiones RDP/VNC, teclas, portapapeles y resize reales.
- Cierre de sesiones interactivas y comportamiento de los clientes frente a fallos de red.
- Pruebas en Ubuntu 24.04, Debian 13 y otras distribuciones; validación Wayland nativa.
- Empaquetado redistribuible para runtimes antiguos; Flatpak/AppImage aún no preparados.

Los artefactos de compilación quedan en `target/`, excluido por `.gitignore`.
El estado de Git y los resultados de apartados posteriores deben consultarse por
separado; esta sección describe únicamente la validación inicial.

## Restauración de pestañas FTP/SFTP — 29 de octubre de 2026

- Referencias a perfil y protocolo, carpeta remota absoluta conocida, orden y
  selección; recuperación desconectada sin consultar credenciales ni iniciar SFTP.
- Conexión explícita con los mecanismos existentes; reemplazo de la pestaña en su
  posición tras preparar el cliente. Un fallo al buscar credenciales conserva la
  pestaña para reintentar; cerrarla durante esa búsqueda cancela la apertura.
- Prueba GTK aislada `python3 lab/check_workspace_transfers.py`: FTP/SFTP,
  conservación de referencias, ausencia de nuevos clientes/master SFTP y rechazo
  de perfiles eliminados o cambiados de protocolo. No contacta servidores reales.
- Prueba unitaria de serialización y rechazo de rutas relativas, caracteres de
  control, protocolos incompatibles y exploradores dentro de divisiones.
- Resultado: compilaciones debug y release correctas; 60 pruebas de biblioteca aprobadas;
  `check_workspace_transfers.py` aprobado, incluido reemplazo de la pestaña con
  canales de transferencia en memoria y conservación de un único explorador
  local; los cuatro escenarios de `check_workspaces.py` también aprobados.
- La autenticación y navegación reales tras recuperar la pestaña requieren aún
  validación con servidores de laboratorio. RDP/VNC y foco/desplazamiento quedan
  fuera de esta entrega. No se han hecho commits ni push.

## Restauración de escritorios RDP/VNC — 29 de octubre de 2026

- Las pestañas de escritorio conservan perfil y protocolo, orden y selección en
  espacios y capturas automáticas. La recuperación no inicia clientes: primero
  se pulsa **Abrir escritorio**, después se introducen credenciales y se pulsa
  **Conectar**. No se serializan contraseñas, frames, eventos de entrada ni
  excepciones de confianza de certificados.
- Un perfil eliminado o cambiado de protocolo permanece como aviso. La apertura
  vuelve a comprobar el perfil actual; conserva la posición de la pestaña y usa
  el visor y sus mecanismos existentes de desconexión y validación TLS.
- `python3 lab/check_workspace_desktops.py`: aprobado el recorrido GTK para ambos
  protocolos, ausencia de conexiones al restaurar/abrir credenciales, orden,
  selección, rechazo de perfiles incompatibles, exclusión de contraseña y cierre.
- `python3 lab/check_vnc.py --workspace --password`: aprobado contra un servidor
  RFB desechable en loopback. Incluye guardar/cargar el espacio, restauración,
  autenticación, framebuffer, teclado, ratón, rueda y cierre real de conexión.
- `cargo test --locked --lib --jobs 2`: 61 aprobadas. Compilaciones debug y release correctas;
  `py_compile` de los dos scripts y `git diff --check` correctos. Regresión de
  espacios y restauración FTP/SFTP comprobada con sus scripts existentes.
- No se ha probado una conexión RDP real restaurada. Foco de paneles y estados
  visuales restantes siguen pendientes. No se han hecho commits ni push.

## Foco, zoom y posición de lectura — 29 de octubre de 2026

- Las hojas de terminal guardan `view` opcional con zoom entre 0,5 y 3 y panel
  activo. Los marcadores desconectados conservan esos valores al volver a guardar;
  el foco se aplica al botón del panel de la pestaña seleccionada y el zoom al
  VTE creado únicamente cuando se solicita abrir la terminal.
- Los chats guardan una posición relativa de lectura opcional. La restauración
  espera la asignación gráfica; si la pestaña está oculta conserva el valor
  pendiente. Las respuestas nuevas no desplazan al lector que está hacia arriba;
  si está al final, mantiene el seguimiento de nuevas respuestas.
- Las capturas antiguas usan zoom 1 y posición final del chat. Se rechazan zoom y
  posiciones fuera de rango. No se guarda scrollback ni desplazamiento de terminal.
- `cargo test --locked --jobs 2`: 87 aprobadas y 5 omitidas (pruebas que requieren
  display o laboratorios externos). Incluye compatibilidad y límites de los nuevos
  campos. Compilaciones debug y release, `py_compile` del script y
  `git diff --check` correctos.
- `python3 lab/check_workspace_visuals.py`: aprobado en GTK/Xvfb; restauración sin
  procesos nuevos, foco de la división seleccionada, zoom aplicado al abrir,
  posición de chat pendiente/visible, lectura estable al añadir contenido y
  seguimiento de respuestas al volver al final.
- `check_workspaces.py` y `check_workspace_chats.py`: aprobados sus cuatro modos
  respectivos, incluida reanudación explícita y fallo recuperable del proveedor.
- No se han hecho commits ni push. Siguen pendientes otros estados visuales de
  editores/exploradores y la validación real SSH/RDP ya documentada.
- `python3 lab/check_chat_state.py`: sus seis escenarios también aprobados
  (API, ACP, reintentos, desconexión, adjuntos/razonamiento y archivo largo), sin
  ejecutar proveedores reales.

## Vistas de editor/exploradores y validación remota — 29 de octubre de 2026

- Campos opcionales y compatibles con capturas anteriores: cursor/ancla en
  caracteres y desplazamiento relativo por editor; ocultos, nombre seleccionado
  y desplazamiento por explorador local, SSH y FTP/SFTP. Se validan posiciones,
  límites y nombres simples. No se activan archivos ni se pagina automáticamente.
- Aplicación tras carga asíncrona del texto/listado. Una carpeta solicitada durante
  la enumeración inicial se conserva y se encadena; callbacks de restauraciones
  anteriores se invalidan. Las vistas ocultas conservan valores pendientes.
  Reutilizar un editor mantiene su vista actual; una selección ausente queda vacía.
- Cada división SSH recuperada conserva la carpeta/vista de su propio panel incluso
  si comparte transporte con otra división. El explorador lateral sigue el foco
  del terminal; no se toma siempre el primer panel del perfil.
- `cargo test --locked --jobs 2`: 88 aprobadas y 5 omitidas por sus requisitos de
  display/laboratorio. Nuevas pruebas de compatibilidad, límites y persistencia.
- `python3 lab/check_workspace_views.py`: aprobado con cursor/selección Unicode,
  lectura tras cargar, restauración durante enumeración, ocultos, selección
  desaparecida, reutilización del editor y listado remoto sintético. Se comprueba
  la posición real del viewport, además del valor pendiente de vistas ocultas.
- `check_workspace_documents.py`: aprobados sus siete escenarios de recuperación,
  guardado, borradores ausentes/dañados y preservación del espacio anterior.
  `check_workspace_transfers.py` y `check_workspace_visuals.py` también aprobados.
- `python3 lab/check_ui.py ssh --workspace`: aprobado contra el laboratorio en
  loopback; guardar/cargar el espacio, restauración sin conexión, conexión explícita,
  dos divisiones con transporte compartido y carpetas distintas, listados reales,
  selección remota, cambio del explorador con el foco y cierre de la conexión.
- `python3 lab/check_ui.py rdp --workspace`: aprobado contra xrdp del laboratorio,
  con framebuffer real, teclado/ratón y cierre. `rdp-certificate --workspace`
  también aprobado: diálogo visible y rechazo del certificado no confiable.
  Al arrancar, el laboratorio regeneró su certificado; se actualizó la CA local
  directamente desde el contenedor con `lab/lab.py trust`, manteniendo validación
  TLS y de nombre. No se modificó la confianza global del sistema.
- `python3 lab/check_file_browser.py sftp`: aprobado con un único explorador local,
  navegación, transferencias, portapapeles y arrastre en ambos sentidos, protección
  contra sobrescritura y cierre. Las dos pruebas explícitas de `shared_sftp` también
  aprobadas (transporte compartido e independiente).
- Compilaciones debug y release correctas, `py_compile` de los scripts modificados
  y `git diff --check` correctos.
- El contenedor propio, inicialmente detenido, se inició para las pruebas y quedó
  detenido de nuevo. No se usaron equipos de producción ni se hicieron commits/push.
- Límite: la selección se aplica al listado cargado; no se reconstruye una página
  posterior automáticamente. Siguen pendientes las fases de transferencias por
  lotes/colas, diff remoto, ejecución supervisada, recetas, túneles e intervenciones.
