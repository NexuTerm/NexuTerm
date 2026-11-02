# Arquitectura de NexuTerm

Revisión documental: 26 de octubre de 2026. Basada en el código del checkout;
no constituye una prueba de ejecución ni de compatibilidad con servidores reales.

## Plataforma

Aplicación Linux en Rust, edición 2024, con Rust mínimo 1.92 según `Cargo.toml`.
La interfaz usa GTK4, libadwaita y VTE. La característica `gui` está habilitada
por defecto; el binario requiere esa característica. La biblioteca permite
comprobar backends con `cargo test --locked --no-default-features`, pero su
compilación también requiere LibVNCClient por el puente C de `build.rs`.

## Organización del código

| Área | Módulos en `src/` | Responsabilidad |
| --- | --- | --- |
| Interfaz | `main.rs`, `ui.rs`, `actions.rs`, `sessions.rs` | Ventana, pestañas, acciones y gestión de perfiles |
| Configuración | `config.rs`, `connection.rs` | Persistencia versionada y validación de conexiones |
| Terminal | `ssh.rs`, `terminal_colors.rs` | OpenSSH, socket de control y paletas VTE |
| Archivos | `file_browser.rs`, `remote_panel.rs`, `transfer.rs`, `sftp_session.rs` | Navegación local/remota y SFTP |
| FTP | `ftp.rs`, `ftp_transfer.rs`, `ftp_legacy.rs` | Integración GVfs y compatibilidad Windows CE mediante curl |
| Editor | `files.rs`, `remote_edit.rs`, `syntax.rs` | Texto UTF-8, conflictos y resaltado sintáctico |
| Escritorio remoto | `desktop.rs`, `desktop_view.rs`, `rdp.rs`, `vnc.rs`, `vnc_bridge.c` | RDP con IronRDP y VNC con LibVNCClient |
| Agentes | `ai_panel.rs`, `ai_chat.rs`, `codex_chat.rs`, `ai_settings.rs`, `ai_config.rs` | Chat gráfico y configuración de proveedores |
| Integración de agentes | `codex.rs`, `agent_profile.rs`, `agent_runtime.rs`, `terminal_mcp.rs`, `ai_terminals.rs` | Transportes, perfiles privados y acceso autorizado a terminales |
| Dictado | `ai_dictation.rs`, `dictation.rs` | Captura local y transcripción mediante motor externo |

La presencia de un módulo no demuestra que todas sus capacidades estén expuestas
en la interfaz. El alcance de uso está en el [README](../../README.md).

## Conexiones y archivos

SSH utiliza el cliente OpenSSH. El navegador de una pestaña SSH reutiliza su
socket de control privado mediante SFTP. La sesión SFTP independiente no abre
un shell remoto. FTP utiliza GVfs y un backend específico para servidores
Windows CE. Telnet se ejecuta dentro de una terminal.

RDP y VNC se muestran en pestañas propias sin exigir un visor externo.
RDP usa IronRDP y TLS del sistema; VNC enlaza LibVNCClient mediante un puente C.

Las transferencias ordinarias rechazan sobrescrituras. SFTP pagina los listados
en grupos de 200 entradas y usa bloques de 64 KiB. El editor limita el texto a
8 MiB y distingue el guardado local con etag del guardado remoto SFTP: temporal
privado, verificación y reemplazo atómico cuando el servidor ofrece POSIX rename.
Se conservan propietario/grupo/permisos POSIX, no se garantizan ACL ni atributos
extendidos. Los borradores recuperados requieren vinculación explícita al mismo
destino y comparación de la huella original antes de habilitar Guardar.

Las hojas de terminal incluyen `view` opcional (zoom acotado y foco), compatible
con capturas antiguas. Se conserva también en los marcadores desconectados; al
abrirlos se aplica el zoom al VTE creado. Solo el panel de la pestaña seleccionada
recibe foco durante la recuperación. Cada chat guarda un desplazamiento relativo
opcional, aplicado tras su asignación gráfica; un chat oculto mantiene el valor
pendiente. El seguimiento del final se suspende cuando el usuario lee hacia arriba.

Los documentos incorporan una referencia de vista opcional (cursor y ancla en
caracteres, desplazamientos relativos), aplicada por el tracker tras cargar el
texto. El explorador principal, las pestañas de archivos y los paneles SSH tienen
vistas opcionales con ocultos, nombre seleccionado y desplazamientos relativos.
La enumeración local pendiente se encadena sin perder el destino solicitado;
las restauraciones nuevas invalidan callbacks gráficos anteriores. Las vistas
ocultas conservan sus valores pendientes y reutilizar un editor no reemplaza su
vista actual. Seleccionar no activa archivos ni provoca paginación adicional.

El esquema de espacios incluye pestañas `FileBrowser` (perfil, protocolo y
carpeta remota) y `Desktop` (perfil y protocolo). Son pestañas completas y se
restauran como referencias desconectadas. La apertura explícita comprueba de
nuevo el tipo de perfil y reemplaza la pestaña en la misma posición; los visores
RDP/VNC muestran sus credenciales antes de conectar. No se guardan conexiones,
contraseñas, frames ni decisiones de confianza de certificados.

Los espacios conservan referencias de documento por UUID y origen, con los
borradores en el almacén privado de recuperación. Antes de guardar se conservan
las ediciones pendientes; un fallo evita sustituir el espacio anterior. Se abren
los documentos en su posición, reutilizando los editores vivos por identidad.
Los documentos SSH limpios esperan una acción explícita y una sesión existente;
los borradores remotos pueden recuperarse sin red.

La paleta de IA expone captura/restauración al gestor de espacios. El JSON añade
un bloque opcional `chats` compatible con los espacios anteriores, con identidades
UUID, metadatos de perfil/modelo/carpeta y copias acotadas de los mensajes visibles
y del borrador. La restauración crea vistas sin cliente ni credenciales y reutiliza
las identidades vivas; comprueba la capacidad antes de tocar la estructura.
Codex publica la identidad nativa al crear o reanudar una conversación. La acción
explícita de reanudación usa esa referencia y bloquea el envío hasta recibir
`Resumed`; la copia local permanece si el proveedor falla. `chat_state` añade
contexto API de turnos completados y referencias ACP ligadas al destino del perfil.
Las API se rehidratan en memoria; ACP difiere `session/new` y negocia `loadSession`
o `sessionCapabilities.resume` antes de cargar. Durante la carga rechaza solicitudes
y omite la reproducción que ya representa la copia local. El envío espera `Continued`.
No se serializan concesiones ni peticiones de aprobación.

`chat_history` conserva un registro incremental JSONL por identidad de chat. Un
actor con cola acotada serializa lotes en un worker; un bloqueo global comprueba
la cuota antes de publicar y sincronizar. Cada mensaje tiene UUID propio y los
fragmentos/reemplazos mantienen el texto completo, separado de la vista acotada.
Los índices visuales son de página y no cargan todo el archivo en memoria. El
borrado publica una marca que impide nuevas escrituras sobre la identidad retirada.
`chat_history_ui` consulta páginas de 25 mensajes y exporta a un archivo local nuevo;
la captura completa se puede excluir y no sustituye a la copia de recuperación.
Las capturas de espacios/sesión drenan las escrituras de historial antes de guardar
su referencia, y el cierre de un chat drena su cola antes de retirar el registrador.

La recuperación automática reutiliza el esquema de espacios en un almacén
separado, `workspaces/sessions/<UUID>.json`. `session_recovery` administra copias
privadas y atómicas y bloqueos `flock` sobre archivos auxiliares estables ante el
reemplazo del JSON. Cada ventana mantiene una identidad y un bloqueo mientras
está abierta. El lector omite sesiones bloqueadas y conserva formatos inválidos.
`session_ui` captura cada dos segundos, conserva antes los borradores y ejecuta
el I/O en workers. Solo hay una escritura en curso por ventana y se omiten
estados sin cambios. El cierre y el relevo de una copia recuperada fuerzan una
escritura; la copia anterior solo se retira después de confirmar la nueva.
La restauración añade estructuras desconectadas, con el mismo comportamiento
seguro del gestor de espacios. Los espacios con nombre no se modifican.

## Configuración y credenciales

La configuración se guarda en `$XDG_CONFIG_HOME/nexuterm/sessions.json`, con
alternativa `~/.config/nexuterm/sessions.json`, esquema versionado y escritura
atómica. Las contraseñas no se almacenan en ese JSON; las credenciales que se
persisten utilizan referencias al llavero Secret Service.

Los agentes usan directorios privados bajo `$XDG_DATA_HOME/nexuterm/agents/`.
Los transportes nativos y las API tienen capacidades diferentes. El acceso a
terminales exige habilitación y aprobación gráfica de las operaciones.

## Distribución y comprobación

`Makefile` y `packaging/release.py` preparan el paquete Debian, comprueban
requisitos de ejecución e instalan el binario, lanzador e icono. `build.rs`
compila los recursos GTK y el puente VNC. Las versiones de bibliotecas y las
instrucciones de instalación están en el [README](../../README.md).

Las pruebas se reparten entre pruebas Rust, `tests/`, ejemplos de diagnóstico y
scripts gráficos de `lab/`. Los resultados fechados y sus limitaciones están en
[VALIDACION.md](../VALIDACION.md); las conexiones reales de IA tienen su
[propio registro](../PRUEBAS_CONEXION_IA.md).

## Cola de transferencias y navegación de chats

`transfer_queue.rs` mantiene el diario privado por identidad de destino, bloqueo
entre procesos, revisión y estados de operación. `transfer_queue_ui.rs` comparte
el controlador entre paneles del mismo destino y guarda antes de enviar al
transporte. `transfer.rs` planifica árboles y publica archivos SFTP; FTP reutiliza
GVfs para navegación y curl con credenciales por stdin para subidas. Ningún
estado recuperado ejecuta una transferencia automáticamente.

`codex_chat.rs` alterna recientes/conversación mediante GtkStack, conservando el
editor fuera de ambas páginas. `ai_panel.rs` sustituye la instancia de conversación
dentro de la misma página del Notebook; su identidad y archivo cambian juntos.
`chat_history.rs` conserva copias de continuación privadas filtradas por agent_id,
con cuota compartida con JSONL, escritura atómica y orden de captura para rechazar
escrituras tardías. El borrado invalida tanto el JSONL como la copia de recientes.

## Resolución de versiones del editor

`editor_compare.rs` mantiene la revisión modal Base/Local/Original y el resultado
manual. Verifica el original antes de adoptar la base; aplicar modifica solamente
el editor y fuerza la conservación del borrador. `text_diff.rs` calcula diferencias
por líneas fuera de GTK, con prefijo/sufijo común, líneas internadas y LCS acotado
a dos millones de celdas; por encima muestra regiones completas sin truncar texto.

`RemoteDocument` conserva texto, huella y metadatos de la versión base. La lectura
estable y la aceptación comparan tamaño, mtime, tipo, UID/GID y permisos; atime no
se toma como conflicto porque leer el archivo puede actualizarlo. La vista de un
borrador recuperado con un original cambiado no autoriza guardar hasta revisar.
El registro privado `Draft` añade `base_text` opcional (8 MiB por base y 8 MiB por
borrador), validado contra la huella SSH. Los registros anteriores siguen siendo
legibles. La base avanza tras guardar o aceptar una revisión, sin reproducir
operaciones ni utilizar otro destino. No existe bloqueo SFTP entre comprobación
y rename y no se promete conservación de ACL/xattrs no expuestos por el cliente.

## Ejecución supervisada

`execution.rs` controla procesos `/bin/sh` con pipes independientes y grupo propio;
SSH reutiliza un `SshConnection` capturado al aprobar y un canal multiplexado sin
alternativa de autenticación. El timeout abarca la preparación SSH y el proceso.
Se drenan stdout/stderr con captura total de 4 MiB; el cancelador es atómico y la
terminación local señala el grupo antes de reaprovechar/recolectar el PID. Los
resultados remotos inciertos usan `Unknown` y retorno ausente. Un semáforo limita
la aplicación a cuatro trabajos; los snapshots de vista se acotan por flujo.

`execution_history.rs` prepara un registro durable antes de lanzar, reserva el
peor tamaño JSON, aplica cuota privada de 1 GiB/1000 archivos, bloquea entre
escritores y publica por temporal/fsync/rename. Las reservas activas evitan borrar
registros en curso. Los temporales no publicados cuentan sin eliminar su contenido.
Se conserva cada segundo como máximo una escritura en curso, y al terminar se
espera la escritura final. Una captura `Preparing/Running` sin propietario activo
se lee como resultado desconocido, sin iniciar procesos.

`supervised_ui.rs` gestiona formulario, revisión, resultados, historial y exportación.
No pausa el guardado automático de espacios durante toda una ejecución. El cierre
normal confirma cancelar, espera y protege resultados no guardados. La memoria de
cada ventana conserva como máximo ocho trabajos y no retira datos pendientes de
conservación sin confirmación. Las lecturas de historial se serializan por ventana
con selección más reciente y descarte de respuestas obsoletas.

El broker existente anuncia `supervised_targets`/`supervised_execute`; el handler
requiere destino y carpeta explícitos y aprobación nueva. Valida que la solicitud
siga viva antes de lanzar y cancela si se cierra la respuesta pendiente. El retorno
MCP contiene colas acotadas, indicadores de truncamiento e identidad de ejecución;
la captura completa disponible queda en el registro/exportación, no se reinyecta
como entrada ni se utiliza para repetir acciones.

### Acceso completo temporal del chat

El selector común conserva Solo lectura y Pedir permiso y añade Acceso completo.
El cliente mantiene el permiso en memoria; Codex recibe políticas explícitas al
crear/reanudar el chat y en cada turno, de modo que la revocación se aplica al turno
siguiente. Las aprobaciones conocidas de comandos, archivos y herramientas se
aceptan cuando el permiso está activo; las preguntas al usuario no se responden
automáticamente. ACP conserva su traducción a opciones de autorización del agente.

Cada conversación dispone de un Broker MCP y un socket privado independientes;
el permiso del puente es una bandera compartida solo por ese chat. El sondeo GTK
termina al desconectarse el canal para no acumular temporizadores al cerrar chats. Terminales y
procesos supervisados consultan esa bandera, manteniendo validación de argumentos,
cuotas, cancelación y comprobación de solicitudes caducadas. Cerrar, cambiar de
conversación o recuperar reinicia el acceso a Solo lectura; no hay campos nuevos
en los formatos persistidos. La configuración de herramientas del perfil y las
restricciones del sistema operativo siguen determinando las capacidades disponibles.

### Recetas de diagnóstico

`recipes.rs` valida parámetros tipados y genera comandos Linux de solo consulta
para disco, servicio y journal. Reutiliza el quoting de SSH, restringe las unidades
y acota ruta, umbral, líneas, periodo y tamaño final del script. Cada receta incluye
una comprobación final cuyo mensaje distingue atención, error y consulta completada.
No hay un motor genérico de workflows ni modificación del formato de Record.

`recipe_ui.rs` añade selector, parámetros y vista previa al formulario supervisado.
La acción Revisar regenera el comando desde los campos validados y la aprobación
captura ese Spec con su Target. Se utiliza el mismo proceso local o canal SSH y
el mismo diario privado. Los callbacks mantienen referencias débiles a los widgets;
el borrador libre se conserva al alternar vistas y nunca se ejecuta al seleccionar.
Las pruebas de comandos usan herramientas sintéticas; las de disco también cubren
GNU df real en local y a través del servidor SSH desechable.

### Importación estática y túneles

`ssh_import.rs` lee texto regular acotado en segundo plano y construye una vista
parcial de alias explícitos. No lanza OpenSSH ni evalúa Includes/Match. Aplica
primer valor y patrones Host, valida los campos de conexión y conserva advertencias
acotadas. `Profile.ssh_ignore_config` tiene valor anterior false por defecto y true
para importados; tanto el lanzamiento inicial como sus canales usan `-F /dev/null`.
La interfaz permite revisar el campo sin alterar perfiles existentes.

`tunnels.rs` valida reglas tipadas y controla forward/check/cancel sobre el socket
privado del ControlMaster. No hay shell remoto ni fallback a conexión nueva. La
tarea mantiene estado, señal de cancelación y referencia a la conexión; destruir
el manejador solicita detener un túnel activo. Los fallos sin acuse de apertura
son desconocidos y no cancelan una regla externa automáticamente. La cancelación
manual de un desconocido es una operación explícita. No hay reconexión ni reintento
automático; se acotan mensajes y operaciones y se comprueba el master periódicamente.

`ssh_tools_ui.rs` reutiliza el guardado privado de Config y el esquema Tunnel
existente. Las definiciones y perfiles se guardan sin activar conexiones. El destino
se valida por identidad de perfil incluso si GTK autoselecciona una fila. Las filas
de resultados conservan el manejador mostrado para que cancelar no dependa de un
índice que pueda cambiar al retirar resultados antiguos. El cierre de ventana
oculta; el cierre de aplicación confirma y espera cancelaciones antes de destruir
las conexiones. Los estados de ejecución no se serializan ni se reactivan al recuperar.

### Intervenciones

`interventions.rs` guarda contextos versionados independientes del historial de
`execution.rs`. El bloqueo `flock`, las escrituras atómicas sincronizadas y la
comparación del borrador esperado protegen frente a concurrencia y pérdida de la
última copia válida. Los archivos se leen con límites, sin seguir enlaces ni
bloquear en archivos especiales; las cuotas cuentan también temporales huérfanos.
No existe un motor de repetición de acciones.

`intervention_ui.rs` conserva objetivo, destinos, notas, exclusiones y borrador;
la selección de contexto no vincula implícitamente otras acciones. La preparación
vinculada abre `supervised_ui.rs`, que captura el contexto y la solicitud exacta
para su revisión. Tras aprobar, se persiste propuesta/aprobación con UUID antes de
llamar a `execution::start_with_id`. Si guardar falla, no se ejecuta; si se cierra
el formulario antes del arranque, la aprobación puede quedar sin resultado confirmado.
El historial conserva su formato y sigue siendo la fuente de estados y salidas.

El informe comprueba UUID, especificación y destino de cada evidencia. Los registros
ausentes o incompatibles no se sustituyen; la salida se limita a muestras señaladas.
La vista previa editable se invalida al cambiar selección u opciones y el exportador
crea un archivo 0600 sin reemplazar existentes. El JSON editado se valida antes de
exportar. La redacción de secretos queda a cargo del usuario.

El cierre normal fuerza el borrador pendiente. Las ventanas retenidas de intervención
y ejecución supervisada se desvinculan y destruyen antes de su padre, evitando
referencias transitorias inválidas en GTK 4.14. El diario de transferencias libera
explícitamente su bloqueo al destruirse: un descriptor heredado por `fork` no puede
prolongar la exclusión del propietario ya cerrado.
