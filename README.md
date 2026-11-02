# NexuTerm

Aplicación Rust para Linux con terminal por pestañas y conexiones SSH, SFTP, Telnet, FTP, RDP y VNC.
Versión 0.2.0 en desarrollo. Consulta los límites conocidos y las comprobaciones de validación antes de distribuirla.

## Primera versión

- Interfaz GNOME con libadwaita: cabecera, barra de iconos con nombres accesibles, menú de acciones y pestañas compactas y reordenables.
- Menús contextuales con clic derecho o Mayús+F10: abrir, copiar/pegar y operaciones básicas en los exploradores; copiar, pegar, seleccionar y buscar en terminales locales y remotas; copiar selección, mensaje o conversación en los chats de IA. El cuadro de escritura conserva el menú nativo de edición de GTK.
- Explorador local predeterminado con un tercio del área disponible y proporción conservada al redimensionar (separador ajustable), vista de detalles Nombre/Tipo/Tamaño/Permisos y filas alternas grises adaptadas al tema. Los paneles se alinean arriba y abajo, con 3 px entre ellos. La navegación y apertura están integradas en su cabecera, sin pie permanente de paginación o contador. Doble clic o Intro navega por carpetas y abre archivos. Sigue los cambios de carpeta de la terminal local activa mediante VTE o `/proc`; navegar por el explorador no ejecuta comandos en el shell. Las conexiones guardadas siguen disponibles en **Administrar sesiones**.
- Lateral ocultable con F9. Preferencias persistentes de tema, fuente, scrollback y visibilidad inicial del lateral.
- Divisiones horizontales y verticales anidadas, búsqueda literal en terminal, zoom y herramientas en la cabecera de la terminal. Reconexión, desconexión y archivos remotos están en **Conexiones**.
- Cada división tiene una **×** para cerrar solo ese panel y su proceso. El panel restante ocupa el espacio disponible; para cerrar la última terminal usa la **×** de su pestaña.
- Paletas de terminal adaptadas al tema: gris carbón en oscuro y marfil en claro, texto, cursor y selección coordinados y colores ANSI suaves. Se actualizan también en terminales abiertas al cambiar el tema del sistema. Las aplicaciones que fijan sus propios colores RGB conservan esos colores.
- Terminal local mediante GTK4/VTE: PTY, Unicode, redimensionado y scrollback de 10.000 líneas.
- Ventana **Administrar sesiones** (Ctrl+Mayús+S): crear, editar, duplicar, eliminar con confirmación, buscar, agrupar y marcar favoritos.
- FTP y SFTP con explorador remoto y el explorador local principal, sin duplicarlo: navegación, subida, descarga, progreso y cancelación. Selecciona archivos o carpetas locales y pulsa Subir; las descargas usan la carpeta del explorador principal. Puedes arrastrar la selección entre ambos exploradores o usar Copiar/Pegar (Ctrl+C/Ctrl+V con el foco en la lista). La cola persistente permite pausar, cancelar y reintentar desde el inicio de cada archivo, sin sobrescribir destinos existentes. FTP configura usuario y contraseña antes de conectar; puede guardarse en el llavero o mantenerse durante la ejecución. FTP y Telnet transmiten sin cifrar.
- Telnet abre una terminal integrada.
- SFTP independiente utiliza OpenSSH con clave, agente o contraseña solicitada en una ventana gráfica. No abre un shell remoto.
- SSH mediante OpenSSH con usuario, puerto, clave y salto; contraseñas solicitadas en el PTY.
- Navegador remoto automático en el lateral de la aplicación para cada sesión SSH: ruta editable, carpeta de inicio, carpeta superior, páginas de 200 entradas, subida/descarga de archivos, progreso y cancelación. Usa SFTP sobre la conexión OpenSSH compartida del terminal, con la misma autenticación, puerto, clave y salto.
- Editor de texto con números de línea y cabecera moderna mediante GIO, UTF-8 hasta 8 MiB, con comprobación etag antes de guardar y aviso de cambios sin guardar. Resaltado por lenguaje (Python, Rust, C#, SQL y otros), con funciones, palabras clave, cadenas y comentarios diferenciados y colores adaptados al tema.
- VNC integrado en una pestaña GTK: escritorio ajustado al área disponible, teclado, ratón y rueda, contraseña temporal y desconexión al cerrar la pestaña. No necesita Remmina ni otro visor.
- RDP integrado mediante IronRDP: escritorio en una pestaña GTK, teclado, ratón, contraseña temporal y TLS con validación de certificado y nombre del servidor. No necesita FreeRDP ni Remmina.
- Configuración XDG versionada, escritura atómica y archivo privado; no contiene campos de contraseña.

En **Preferencias → Navegador** puedes configurar las columnas Tipo, Tamaño y
Permisos (Nombre siempre visible), si se muestran los ocultos al abrir, la
visibilidad inicial del lateral y la aplicación para abrir archivos locales.
Los cambios se guardan y se aplican a los paneles abiertos, locales y remotos.
Los permisos se muestran como `rwxr-xr-x`, incluyendo setuid/setgid/sticky;
si el servidor no los proporciona se muestra `—` en lugar de inventarlos.

**Ctrl+H**, con el foco en el explorador, y el botón del ojo muestran u ocultan
los nombres que empiezan por punto, sin cambiar el atajo de la terminal. Los
botones **Nueva carpeta** y **Nuevo archivo** piden un nombre simple y crean en
la carpeta actual sin sobrescribir destinos existentes. Están disponibles en
local y SSH/SFTP, y en FTP cuando el backend del servidor lo admite; los
servidores FTP Windows CE heredados no ofrecen creación desde este navegador.
La barra vertical reserva su propio espacio y las columnas que no caben tienen
desplazamiento horizontal compartido con sus cabeceras.

En **Preferencias** puedes elegir abrir los archivos del explorador con la aplicación predeterminada del sistema en lugar del editor integrado, y mostrar u ocultar los números de línea (se aplica a nuevos editores). Por defecto se usa el editor integrado con numeración. Los archivos binarios o que no sean UTF-8 no se editan.

El archivo de configuración está en `$XDG_CONFIG_HOME/nexuterm/sessions.json` o `~/.config/nexuterm/sessions.json`.
Si no se puede cargar o contiene una versión desconocida, se conserva y se deshabilita el guardado de sesiones.

## Importación de perfiles OpenSSH

En **Menú → Conexiones → Importar perfiles OpenSSH…** selecciona un archivo local
para previsualizarlo. No se ejecuta `ssh -G`, no se evalúa `Match` y no se recorren
archivos `Include`. Se muestran avisos de las directivas omitidas sin ejecutar ni
copiar sus órdenes. El archivo original no se modifica y no se abre una conexión.

La importación es parcial y estática: alias `Host` explícitos, `HostName`, `User`,
`Port` y primer `IdentityFile`. Respeta el primer valor aplicable y la herencia de
patrones `Host` con `*`, `?` y exclusiones `!`; los patrones no crean perfiles.
Admite comillas, comentarios y `clave=valor`. Las claves requieren ruta absoluta
o `~/`; no se expanden variables ni tokens de identidad. `HostName` admite `%h`
para el alias. `ProxyJump`, `ProxyCommand`, reenvíos y las demás directivas se
avisan y omiten: revisa los campos antes de importar una configuración parcial.

Marca los perfiles deseados y pulsa **Importar seleccionados sin conectar**. Los
nombres ya existentes no se sobrescriben. Los nuevos perfiles usan
**SSH: ignorar configuración externa (-F /dev/null)**, también en canales
multiplexados, para no ejecutar indirectamente lo que se omitió en la vista
previa. Puedes revisar ese ajuste en la organización del perfil al editarlo.
Los perfiles anteriores conservan su comportamiento. No se copian claves privadas
ni credenciales: solo la ruta de identidad. Límites: archivo regular UTF-8 de
1 MiB, 8192 líneas de hasta 16 KiB y 256 alias explícitos.

## Túneles SSH supervisados

En **Menú → Conexiones → Túneles SSH…** selecciona perfil, tipo y extremos:

- **Local (-L):** escucha en este equipo y conecta al destino desde el servidor SSH.
- **Remoto (-R):** escucha en el servidor y conecta al destino desde este equipo.
- **SOCKS local (-D):** proxy SOCKS4/5; cada cliente elige su destino.

La escucha inicial es `127.0.0.1`; admite direcciones IP literales IPv4/IPv6 y
puertos de 1 a 65535. Una escucha fuera de loopback se señala en la aprobación.
Para reenvíos remotos, la política `GatewayPorts` del servidor determina la
interfaz efectiva y puede imponer una escucha distinta a la solicitada.

**Guardar nueva definición** conserva los parámetros en la configuración privada,
incluso sin conexión. Seleccionar una definición o recuperar la sesión nunca la
activa. Para **Revisar y activar…** necesitas una sesión SSH autenticada de ese
perfil; actualiza el listado después de abrirla. La confirmación muestra la sesión
y ambos extremos. Un perfil ausente no se sustituye silenciosamente por otro.

Se utilizan `ssh -O forward/check/cancel` sobre el ControlMaster existente, sin
comando remoto, nueva autenticación ni reconexión automática. La interfaz muestra
Preparando, Activo, Deteniendo, Detenido, No iniciado o Resultado desconocido.
**Activo** confirma la respuesta de SSH, no la disponibilidad del servicio destino.
Un puerto ocupado o una respuesta perdida no se presenta como una apertura segura;
no se cancela automáticamente una regla posiblemente ajena tras un fallo de apertura.

**Detener túnel seleccionado** solicita la cancelación de la regla exacta. Si el
estado es desconocido permite una nueva cancelación explícita; comprueba el destino
antes de repetir una apertura. La cancelación de la escucha puede dejar continuar
conexiones ya establecidas. Cerrar la ventana de túneles solo la oculta; cerrar
NexuTerm pide confirmación, cancela y espera los túneles antes de cerrar las sesiones.

Máximo de 64 definiciones y 16 resultados activos/no confirmados por ventana;
los resultados terminados pueden retirarse al incorporar otros. Se comprueba el
ControlMaster cada dos segundos y cada operación tiene un máximo de cinco segundos.
Las definiciones no contienen credenciales y no tienen autoinicio. Los estados y
mensajes de esta ventana son temporales; se guardan las definiciones, no un diario
de tráfico ni resultados del servicio. Borrar una definición exige confirmación
y no detiene un túnel que siga activo.

## Intervenciones

En **Menú → Vista y herramientas → Intervenciones…** crea un contexto con título,
objetivo y destinos. Añade notas de diagnóstico, propuesta, aprobación, cambio o
verificación; una nota de aprobación documenta una decisión, pero no concede permisos.
El borrador se guarda cada segundo y al cerrar normalmente. Recuperar un contexto
muestra sus notas y evidencias sin repetir comandos ni conectar destinos.

**Preparar ejecución vinculada…** abre el formulario supervisado para diagnóstico,
cambio o verificación. Revisa allí destino, carpeta, comando y tiempo; cada acción
requiere su aprobación habitual y conservar historial. La propuesta y aprobación
exactas se guardan antes de iniciar el proceso. **Desvincular** permite volver al
uso independiente. Los chats y MCP no se vinculan automáticamente al contexto.
También puedes asociar explícitamente una ejecución que ya exista en el historial.
Si falta su registro o quedó interrumpido, el informe muestra la incertidumbre;
nunca intenta recrear la evidencia ejecutando otra vez el comando.

Genera un informe **Markdown o JSON**, revisa su vista previa editable y exporta a
un archivo nuevo. Inicialmente no incluye comandos ni salidas; puedes activarlos
y excluir entradas completas. Revisa y elimina información sensible antes de
exportar: no hay detección automática de secretos. El borrador no se exporta.
Las salidas del informe son muestras de hasta 64 KiB por flujo, con indicación de
truncamiento; el historial de ejecución permite exportar su captura completa.
Cambiar contexto, exclusiones u opciones invalida la vista previa anterior.

Los registros se guardan privadamente en `$XDG_DATA_HOME/nexuterm/interventions`
(o `~/.local/share/nexuterm/interventions`), con directorio 0700 y archivos 0600.
Límites: 64 contextos, 512 entradas y 2 MiB por contexto; 130 MiB y 1024 entradas
de directorio contando escrituras incompletas. Al alcanzar un límite se informa
del error y se conserva lo anterior, sin expulsión automática. Un informe tiene
un máximo de 8 MiB y no sobrescribe archivos existentes. Borrar un contexto requiere
confirmación; no borra el historial de ejecuciones ni cancela procesos en curso.
Los cambios concurrentes del borrador se detectan en lugar de sobrescribirlos.

## Ejecución supervisada

En **Menú → Vista y herramientas → Ejecución supervisada…** prepara un comando,
selecciona **Este equipo** o una sesión SSH abierta y escribe la carpeta absoluta.
**Actualizar destinos** recoge las sesiones SSH abiertas después de mostrar la
ventana. Cambiar de destino vacía la carpeta para que la indiques expresamente.
**Revisar ejecución…** muestra solicitante, destino, carpeta, comando, límite de
tiempo y conservación del historial; **Aprobar y ejecutar** autoriza esa ejecución
concreta. Rechazar o cerrar la aprobación no inicia ningún proceso.

En esa misma ventana, el selector **Comando libre / Espacio en disco / Estado de
servicio / Registros systemd** prepara recetas de diagnóstico de solo consulta:

- **Espacio en disco:** ruta absoluta y umbral de uso entre 1 y 100 % (90 % inicial).
  Muestra `df -Pk` y comprueba el porcentaje final con GNU `df --output=pcent`.
  Un uso igual o superior al umbral devuelve 2 y se marca `ATENCION` en la salida.
- **Estado de servicio:** nombre explícito terminado en `.service`; muestra
  LoadState, ActiveState, SubState y Result, y comprueba `systemctl is-active`.
  Un servicio activo devuelve 0 y uno no activo, 3. Unidades ausentes o errores de
  consulta no se presentan como un servicio sano.
- **Registros systemd:** unidad `.service`, máximo de 1–5000 líneas y periodo de
  1–10080 minutos (inicialmente 100 líneas/60 minutos). Usa `journalctl` sin paginador
  y con fechas ISO. Retorno 0 confirma la consulta de los registros accesibles al
  usuario, no la salud del servicio ni acceso al journal completo.

Las recetas validan los parámetros y muestran una vista previa no editable del
comando. **Revisar ejecución…** vuelve a validarlos y permite aprobar o rechazar
el destino, carpeta y comando exactos. Elegir una receta no ejecuta nada. Al volver
a **Comando libre** se recupera el borrador manual. Las unidades admiten letras
ASCII, números, `_`, `.`, `@`, `:` y `-`, con inicio alfanumérico y hasta 255 bytes;
no se interpretan patrones ni fragmentos de shell.

Funcionan en local y sobre una sesión SSH autenticada de un destino Linux con las
herramientas correspondientes; no instalan programas, solicitan sudo ni cambian
servicios. Heredan timeout, cancelación, límites, historial privado, exportación y
recuperación sin repetición. El historial conserva el comando con la identificación
de receta y sus comprobaciones en la salida. Errores de consulta devuelven 1 y
herramientas ausentes, 127. Un timeout, cancelación o resultado SSH desconocido
mantiene el significado de la ejecución supervisada.

Los comandos se ejecutan mediante `/bin/sh`, con entrada estándar cerrada y
stdout/stderr separados. En local se crea un proceso y grupo propios, con la
carpeta aprobada y el entorno exportado de NexuTerm. En SSH se utiliza un canal
independiente de una conexión autenticada ya abierta; requiere `/bin/sh` remoto y
no inicia otra autenticación si desaparece la conexión compartida. El shell
interactivo conserva su entrada, carpeta y variables privadas.

El tiempo inicial es **60 segundos**, configurable entre 1 y 120. Se capturan hasta
**4 MiB entre stdout y stderr**, drenando ambos flujos aunque se alcance el límite.
La vista muestra hasta 128 KiB por flujo; **Exportar salida…** conserva todos los
bytes capturados en un archivo local nuevo, con cabecera de estado y separación de
flujos. No reemplaza archivos existentes. El resultado indica estado, duración,
código de retorno cuando se confirma y truncamiento de captura. Un retorno distinto
de cero se muestra como finalización con error.

**Cancelar proceso seleccionado** solicita la terminación del grupo local. En SSH,
la cancelación cierra el cliente/canal, pero no garantiza detener el programa del
servidor. Cancelación, timeout, pérdida del canal o retorno SSH ambiguo 255 producen
**Resultado desconocido** cuando no puede confirmarse el resultado remoto, sin
inventar un retorno. Comprueba el servidor antes de repetir esa operación. Los
procesos que se independicen expresamente de su grupo local requieren su propio
control de ciclo de vida.

La ventana conserva hasta ocho resultados en memoria y admite un máximo de cuatro
ejecuciones simultáneas en la aplicación. **Retirar salida de memoria…**, con
confirmación, libera un resultado terminado sin borrar su registro en disco. Las
salidas cuyo historial haya fallado se retienen hasta retirarlas explícitamente;
se pueden exportar antes de hacerlo. Cerrar la ventana de ejecuciones permite
consultarla de nuevo sin cancelar los procesos. Cerrar NexuTerm confirma la
cancelación de los activos y espera al resultado y a su escritura; si no se pudo
conservar el historial, permite cancelar el cierre para exportar o cerrar
expresamente sin ese historial.

**Conservar historial local**, activado inicialmente, guarda comando, destino,
carpeta, estado y salida en `$XDG_DATA_HOME/nexuterm/executions/`, con directorio
0700 y archivos 0600. Se prepara el registro antes de ejecutar y se publican
capturas periódicas y finales mediante reemplazo atómico y fsync. Hay un límite de
**1 GiB y 1000 registros/archivos de historial**, reservando espacio para la captura
máxima de cada ejecución; los temporales incompletos también cuentan. Al faltar
cuota o fallar la preparación no se ejecuta el comando. No se borran registros
antiguos automáticamente. **Historial…** permite consultar, exportar y eliminar
registros confirmando la eliminación, incluso si un registro está dañado. No se
elimina un registro todavía activo. Desmarca la conservación antes de aprobar para
mantener esa ejecución únicamente en memoria; no se serializa el entorno ni se
almacenan credenciales de conexión.

Tras una terminación inesperada se consulta la última captura publicada. Un
registro sin final confirmado se presenta como desconocido; abrir el historial o
recuperar un espacio no repite ni reanuda comandos. Si falla una escritura durante
la ejecución, se avisa y se mantiene la captura acotada en memoria para exportarla;
la escritura final puede confirmar el registro si el almacenamiento se recupera.

Los agentes con acceso MCP a terminales tienen también **`supervised_targets`** y
**`supervised_execute`**. La segunda exige `target_id`, `directory` absoluta y
`command`; `timeout_seconds` y `keep_history` son opcionales. Solicita una
aprobación nueva salvo que el usuario haya elegido **Acceso completo** para ese
chat; no reutiliza concesiones de otros tipos de operación. Una
solicitud caducada antes de aprobar no ejecuta nada; si el solicitante se desconecta
con el proceso activo, se solicita su cancelación. La respuesta incluye estado,
duración, retorno opcional, últimos 4 KiB por flujo de la captura conservada e
indicadores separados de truncamiento de captura/respuesta y de conservación del
historial. `terminal_execute` conserva su comportamiento de envío al shell abierto.

## Espacios de trabajo

En **Menú → Conexiones → Espacios de trabajo…** puedes guardar las pestañas de
terminal y editor actuales, su orden y selección, divisiones y proporciones,
carpeta de cada terminal local, perfiles SSH referenciados, rutas del explorador
SSH, documentos, borradores, chats, visibilidad de los laterales y notas. **Nuevo** prepara otro espacio; **Guardar pestañas actuales**
crea o actualiza el seleccionado. **Guardar nombre y notas** conserva la
disposición guardada. La eliminación pide confirmación y no borra archivos del
proyecto.

**Abrir** añade las pestañas sin cerrar el trabajo existente. Los paneles de
terminal se restauran desconectados: pulsa **Abrir terminal** o **Conectar** en
cada uno.
No se reproduce el contenido del terminal ni sus comandos anteriores; el shell
local comienza en la carpeta guardada, si sigue disponible. SSH reutiliza el
perfil actual y su autenticación habitual; no conserva contraseñas ni sockets.
Una carpeta local ausente o un perfil eliminado no se sustituyen por otro destino.

Las pestañas FTP/SFTP también conservan su orden, perfil, protocolo y última
carpeta remota absoluta conocida. Se recuperan desconectadas y **Conectar** usa
el perfil actual y su autenticación habitual. Un perfil eliminado o cuyo
protocolo haya cambiado no se sustituye por otro. La pestaña conectada ocupa la
posición de la recuperada y reutiliza el explorador local principal; no se
guardan contraseñas, transferencias pendientes ni conexiones activas.

Las pestañas RDP/VNC conservan su perfil, protocolo, orden y selección. Se
recuperan desconectadas: **Abrir escritorio** muestra la interfaz habitual y
**Conectar** inicia la conexión con las credenciales introducidas entonces.
No se guardan contraseñas, imágenes del escritorio, entradas de teclado/ratón ni
excepciones de confianza de certificados. Un perfil eliminado o que cambie de
protocolo conserva un aviso sin abrir otro destino. Se mantienen las
comprobaciones TLS y de certificados existentes de RDP.

Cada panel de terminal conserva su zoom (entre 0,5 y 3) y el panel activo de
la pestaña recuperada recibe el foco en su botón de apertura. El zoom se aplica
al iniciar explícitamente esa terminal. Los chats conservan su posición relativa
de lectura; las respuestas nuevas respetan esa posición si estás leyendo hacia
arriba y siguen el final si ya estabas allí. Las capturas anteriores usan zoom
normal y abren el chat al final. No se recupera el contenido ni el desplazamiento
del terminal, porque no se reproduce su salida anterior.

Los editores conservan el cursor, la selección y el desplazamiento vertical y
horizontal. Se aplican después de cargar el texto, con posiciones acotadas al
contenido actual; un editor ya abierto conserva su vista y sus cambios actuales.
Los exploradores local, SSH y FTP/SFTP conservan ocultos, selección y
posición de lectura después del listado. La selección se recupera únicamente si
el nombre existe entre las entradas cargadas; no se abren archivos ni se recorren
páginas adicionales automáticamente. Mientras una vista permanece oculta se
conserva su desplazamiento pendiente hasta que vuelva a mostrarse.

El almacenamiento está separado de `sessions.json`, en
`$XDG_DATA_HOME/nexuterm/workspaces/workspaces.json` (por defecto
`~/.local/share/nexuterm/workspaces/workspaces.json`), con directorio 0700,
archivo 0600 y reemplazo atómico. Un archivo dañado o con una versión desconocida
se conserva y bloquea la gestión hasta que se resuelva el error.

Los documentos se identifican por editor, con su origen y la referencia a su
borrador. Al guardar el espacio se conservan antes los borradores modificados;
si falla esa escritura, no se sustituye el espacio anterior. La escritura del
espacio se realiza en segundo plano y se impiden operaciones simultáneas en ese
diálogo. **Guardar nombre y notas** no altera sus documentos ni disposición.

Al abrir un espacio:

- Los documentos locales limpios se abren con el contenido actual del original.
- Si existe un borrador de ese editor, se recupera su texto como modificado,
  incluso si el original cambió o desapareció. El guardado mantiene las
  comprobaciones de versión descritas en la sección de recuperación.
- Los borradores SSH se recuperan sin conexión. Un documento SSH limpio espera
  **Abrir original**, que requiere una terminal ya autenticada del mismo perfil
  y destino; no inicia otra conexión.
- Los documentos nuevos se colocan en el orden guardado y se mantiene la pestaña
  seleccionada. Si ya está abierto el mismo editor, se reutiliza conservando su
  posición y sus cambios actuales, sin sustituirlos por un borrador antiguo.
- Si falta un borrador que estaba pendiente, se conserva un aviso y se ofrece
  **Abrir original** expresamente. Puede ocurrir si después de guardar el espacio
  se guardó o descartó ese borrador. No se presenta el original como texto
  recuperado ni se recrean automáticamente archivos ausentes.
- Un borrador dañado, desconocido o de otro origen se conserva sin sobrescribir.
  Abrir su original crea otro editor independiente, de modo que cerrarlo no
  elimina el registro que falló.

El espacio conserva referencias al borrador actual de cada editor; no duplica su
texto ni crea versiones congeladas del contenido. El borrado del espacio no
elimina documentos, originales o borradores. Se siguen admitiendo los espacios
anteriores de terminales.

Los chats de la paleta se guardan con identidad estable, orden, selección,
visibilidad, agente referenciado, modelo, carpeta, mensajes visibles y texto aún
sin enviar. Abrir el espacio los recupera sin ejecutar agentes, acceder a claves
ni transmitir mensajes. Si un chat ya está abierto, conserva su posición y su
contenido actual. Si la suma supera ocho chats, se rechaza la apertura completa
antes de añadir pestañas; hay que cerrar alguno primero. Un perfil eliminado no
impide consultar su copia local.

La reanudación siempre es explícita y espera confirmación antes de permitir
mensajes. Codex conserva su referencia nativa. Las API conservan el contexto del
último intercambio completado, incluidos resultados de herramientas ya recibidos;
restaurarlo no realiza peticiones al proveedor ni repite operaciones. El siguiente
mensaje del usuario utiliza ese contexto con el perfil actual del mismo destino.
Los agentes ACP utilizan `session/load` o `session/resume` únicamente si anuncian
esa capacidad. La carga no crea otra sesión ni ejecuta herramientas reproducidas.
Un fallo conserva la copia y permite **Reintentar reanudación**.

Se recuperan las rutas de hasta 20 adjuntos pendientes y el razonamiento elegido.
Restaurar rutas no lee archivos ni recrea los que falten. Tras enviar el mensaje se
retiran los adjuntos pendientes como antes. El razonamiento se selecciona entre
los valores que anuncia el proveedor; si ya no está disponible se usa Auto con
aviso. Si el destino configurado cambió o faltan referencias antiguas, la copia
permanece consultable y no se reenvía a otro proveedor. No se serializan claves,
concesiones de permisos ni solicitudes de aprobación.

El espacio y la copia de sesión conservan hasta 128 tarjetas visibles por chat y
el borrador, con los límites de ocho chats, 64 KiB de borrador y 16 MiB por archivo
de espacios. El contexto API recuperable admite hasta 2 MiB/129 entradas para
conservar también la última respuesta; los límites de envío al proveedor siguen
siendo 256 KiB/128 entradas por solicitud. La posición de lectura se conserva. El cliente nativo mantiene su límite de 128
elementos por turno; si se supera, interrumpe con un aviso en el historial.

Al abrir un chat se muestra **Chats recientes**, que ocupa toda el área de la
conversación y mantiene el editor visible debajo. Los recientes pertenecen al
perfil de agente seleccionado: Codex, GitHub y los demás perfiles conservan
listas independientes. Seleccionar una conversación la abre y solicita su
continuación en la misma pestaña; escribir desde recientes inicia una conversación
nueva. El botón **Mostrar chats recientes** vuelve a esa vista y conserva el chat
anterior. No crea otra pestaña. Si el agente está respondiendo, la respuesta
termina en segundo plano; el editor y la selección de conversaciones esperan a
que finalice, como el editor de la conversación. El historial nativo de Codex
mantiene su paginación.

Las referencias y el contexto de recientes se guardan junto al historial local,
en copias privadas de hasta 16 MiB, con los últimos 128 mensajes, borrador,
adjuntos y contexto recuperable. Comparten el límite global de 1 GiB/1000
conversaciones con los JSONL. No se guardan claves ni autorizaciones. Los perfiles
eliminados o cuyo destino haya cambiado no se sustituyen por otros. Restaurar un
espacio de trabajo sigue dejando el chat desconectado hasta su reanudación explícita.

El botón **Historial local completo** de la cabecera consulta todas las
conversaciones archivadas, incluidas las cerradas. El archivo separado en
`$XDG_DATA_HOME/nexuterm/chat-history/` conserva mensajes, fragmentos de respuestas
y resultados visibles desde esta versión, aunque ya no estén entre las 128
tarjetas. Se escribe por lotes en segundo plano, en archivos privados 0600 y
directorio 0700. La vista pagina 25 mensajes y acota el texto mostrado; **Exportar
JSONL…** conserva todas las entradas y fragmentos, sin reemplazar archivos
existentes. El registro es un historial de texto, no una cola de acciones.

El límite inicial global es 1 GiB y 1000 archivos de conversación, con entradas
individuales de hasta 2 MiB y una cola acotada (64 entradas y 4 MiB pendientes por chat; importación inicial
acotada a 16 MiB). El lector del agente espera cuando el disco va más lento.
Al agotarse el espacio o fallar la
escritura se avisa y se detiene la captura; no se eliminan datos antiguos ni se
bloquea la conservación de documentos o de la copia de recuperación del chat.
Puedes desactivar **Conservar historial completo de este chat** o **Borrar
historial…**, con confirmación. Esa opción afecta al archivo completo; las copias
de recuperación de los últimos mensajes y del borrador se gestionan aparte. Un
historial borrado no se recrea por escrituras tardías; abre otro chat para iniciar
otro registro. Las copias antiguas solo pueden aportar el texto y las referencias
que realmente conservaron; no se inventa el contexto que faltaba.

Las pestañas FTP/SFTP y RDP/VNC ya se integran en los espacios, junto con el zoom
y el panel activo de terminal. No se conserva el scrollback de terminal ni se reconstruyen páginas de
listado que aún no estén cargadas. Los siguientes pasos están en
[la hoja de ruta](docs/MEJORAS.md).

## Recuperación automática de la sesión

NexuTerm conserva una copia por ventana de las pestañas y divisiones compatibles,
los documentos y sus referencias a borradores, la selección, las carpetas, la
visibilidad de los laterales y los mensajes visibles y borradores de los chats.
No hace falta guardar antes un espacio con nombre. Cada dos segundos, si no hay
otra captura ni una escritura o transferencia en curso, conserva los borradores
y actualiza la copia cuando el estado cambia. El trabajo de disco se realiza en
segundo plano, con una sola escritura de sesión por ventana.

Al arrancar se ofrecen las sesiones anteriores. También están en **Menú →
Conexiones → Recuperar sesión anterior…**. **Recuperar** añade el trabajo sin
cerrar las pestañas actuales: las terminales esperan una acción explícita y los
chats se abren sin conectar al agente ni transmitir contenido. Se aplican los
mismos límites y comprobaciones que al abrir un espacio. La copia anterior solo
se retira después de conservar el estado recuperado en la copia de la ventana
actual. **Descartar copia** pide confirmación y no borra originales ni borradores
de documentos. Las copias dañadas o desconocidas se conservan sin modificar.

Las copias están en `$XDG_DATA_HOME/nexuterm/workspaces/sessions/` (por defecto
`~/.local/share/nexuterm/workspaces/sessions/`), separadas de los espacios con
nombre. Se usan directorios 0700, archivos 0600, reemplazo atómico y un bloqueo
por ventana; una sesión activa no se ofrece a otra instancia. Se conservan como
máximo 32 copias pendientes, con un límite de 16 MiB por copia. Al alcanzar el
límite se avisa y no se eliminan copias antiguas automáticamente: recupera o
descarta alguna para permitir nuevas capturas.

El cierre normal fuerza una última copia, incluso antes del primer intervalo.
Si falla, puedes cancelar el cierre o cerrar expresamente sin actualizarla; los
borradores ya conservados permanecen. Un corte puede perder cambios posteriores
a la última captura completada, y las capturas se posponen mientras haya una
escritura o transferencia. No se restauran procesos, comandos ejecutados,
aprobaciones, transferencias ni túneles; tampoco los tipos de pestaña y estados
visuales que siguen pendientes. Se conserva la última copia también tras un
cierre normal, para poder recuperar el trabajo en el siguiente arranque.

Prueba aislada de persistencia y restauración entre procesos:

```bash
cargo build --locked
python3 lab/check_workspaces.py
python3 lab/check_workspace_documents.py
python3 lab/check_workspace_chats.py
python3 lab/check_chat_state.py
python3 lab/check_session_recovery.py
```

Requiere Xvfb y D-Bus y utiliza exclusivamente configuración y datos temporales.
Estos scripts fijan X11 para utilizar Xvfb aunque la sesión del escritorio tenga
Wayland; no cambian la configuración de la aplicación normal.

## Recuperación de documentos

El editor conserva automáticamente los documentos modificados en
`$XDG_DATA_HOME/nexuterm/recovery/` (por defecto
`~/.local/share/nexuterm/recovery/`). Guarda el texto completo, con el límite
habitual de 8 MiB, en archivos privados 0600 dentro de un directorio 0700,
mediante reemplazo atómico y sincronización a disco. No añade credenciales de
conexión al borrador ni envía su contenido a una IA o a un servidor.

La captura se realiza cada segundo, con una única escritura en curso por
editor. Una caída puede perder las ediciones posteriores a la última captura
completada. Al cerrar normalmente, NexuTerm espera a conservar la última
edición; si no puede escribir el borrador, mantiene la ventana abierta e informa
del error. Cerrar la aplicación conserva los borradores. **Descartar** un
editor los elimina expresamente, con confirmación.

Al arrancar se ofrece una lista de documentos recuperables, también accesible en
**Menú → Vista y herramientas → Recuperar documentos…**. Cerrar esta lista no
borra nada. Los borradores dañados o con versión desconocida se conservan y se
indica su número; no se intentan interpretar como documentos válidos.

- **Recuperar** abre el texto como documento modificado sin sobrescribir nada.
- En un documento local, **Guardar** solo se habilita si el original sigue siendo
  un archivo regular con el mismo etag de la versión editada. El guardado vuelve
  a comprobar ese etag. Si cambió o desapareció, conserva el borrador y ofrece
  comparación o copia; no fuerza la sobrescritura.
- **Comparar original** muestra ambos textos en columnas de solo lectura. Para
  un original SSH requiere una terminal autenticada del mismo perfil y destino;
  no conecta ni utiliza otra cuenta automáticamente.
- **Guardar copia…** exporta a un archivo local nuevo, sin reemplazar destinos
  existentes. Conserva el borrador y el estado modificado, porque el original
  sigue pendiente de guardar o descartar.

Los borradores SSH sobreviven a la pérdida de la copia temporal, la conexión o
el perfil. Se recuperan sin conexión y permiten comparar o exportar una copia
local. Para volver a guardar en el servidor, conecta una terminal SSH al mismo
perfil y destino y pulsa **Vincular a SSH**. Se comprueba la huella SHA-256 del
original capturada al editar; si cambió, el vínculo permite comparar, pero
**Guardar** permanece bloqueado hasta revisar y aplicar una resolución. No se
sobrescribe al vincular.
Vincular conserva el texto del editor y no escribe en el servidor. **Guardar**
vuelve a comprobar el original antes del reemplazo atómico; **Revalidar SSH**
permite utilizar otra conexión autenticada del mismo destino si es necesario.

Los borradores de la entrega anterior que no contienen esa huella siguen
recuperándose, pero no permiten vinculación: conservan comparación y copia local.
Después de guardar, la huella se actualiza para recuperar ediciones posteriores.
No se conecta a SSH ni se suben borradores automáticamente.

Un guardado correcto del original retira su borrador si no quedan ediciones
posteriores; estas se conservan en la siguiente captura. No hay caducidad ni
borrado automático de borradores pendientes. Se conserva el límite de 8 MiB;
el guardado remoto utiliza el reemplazo atómico SFTP descrito más abajo.

```bash
cargo build --locked
python3 lab/check_recovery.py
```

La prueba usa Xvfb, D-Bus y datos temporales: incluye `SIGKILL` real, recuperación
entre procesos, cambios externos, original ausente, borrador remoto sin conexión,
descarte confirmado y bloqueo del cierre cuando falla el almacenamiento.

## Agentes de IA y chat

El botón **Agentes de IA** de la cabecera, el menú o **Ctrl+Mayús+I** abre una
paleta lateral derecha redimensionable, independiente de las conexiones.
En **Configurar agentes de IA** puedes añadir perfiles, editar el seleccionado o
eliminarlo con confirmación. El selector y la vista de configuración agrupan las
conexiones en **Locales** y **Web**. El botón **+** de la configuración añade un
modelo mediante el formulario de conexión existente. El formulario limita el motor a
los protocolos compatibles con la conexión y muestra sus parámetros aplicables.
Las conexiones API no muestran ejecutable ni extensiones MCP externas; las nativas
no muestran URL, límite de tokens ni espera de la API. Para Ollama local se consulta
el soporte de herramientas anunciado por el modelo antes de habilitar acceso a
terminales. En proveedores sin metadatos de capacidades, no se deducen capacidades
adicionales únicamente a partir del nombre del modelo.

Todas las plantillas abren el mismo chat gráfico. Elegir una plantilla y guardar
crea una conexión independiente, la añade al selector y abre su chat; no sustituye
al agente anterior ni abre una consola. El selector inferior contiene los modelos
del agente activo, obtenidos del proveedor o añadidos en su configuración.

- **Codex** utiliza `codex app-server`, con streaming, historial y aprobaciones.
- **OpenCode** utiliza `opencode acp`, con catálogo de modelos y permisos ACP.
- **Claude Code** requiere el adaptador `claude-agent-acp` instalado.
- **Gemini CLI** utiliza `gemini --experimental-acp`.
- **GitHub Copilot** utiliza `copilot --acp`.
- **ACP personalizado** permite configurar otro ejecutable compatible.
- **OpenAI Responses** y **Chat Completions** conectan por API. Hay plantillas
  para DeepSeek, Gemini API, Ollama y servidores compatibles. Las respuestas API
  se muestran al completarse; las conexiones nativas pueden transmitir fragmentos.

Los ejecutables y adaptadores deben estar instalados. La compatibilidad con modelos,
historial, permisos y autenticación depende de las capacidades del proveedor.
OpenCode permite actualizar su catálogo gratuito sin sustituir automáticamente un
modelo fallido por otro de pago. Consulta sus condiciones y disponibilidad en
[OpenCode](https://opencode.ai/docs/).

El botón **+ Nuevo chat** permite elegir otro agente y uno de sus modelos configurados.
Abre una pestaña independiente y conserva los mensajes, borrador y conexión de las
conversaciones anteriores. Si el catálogo aún no se ha cargado, se puede elegir el
modelo predeterminado y cambiarlo después en el editor. Se pueden mantener hasta ocho
chats abiertos; cerrar una pestaña adicional detiene únicamente su conexión.

Al abrir un chat, la carpeta de la terminal local activa tiene prioridad sobre la
carpeta configurada en el perfil, sin modificar ese perfil. Cada mensaje vuelve a
capturar el directorio y la terminal actuales. Codex actualiza también el `cwd` de
cada turno, conservando la conversación. Los agentes ACP reciben el directorio al
crear su sesión y los cambios posteriores como contexto explícito del mensaje;
las API reciben ese mismo contexto. Una ruta SSH se identifica como remota y no
se utiliza como carpeta local del proceso. Si no hay una terminal activa, se
conserva la carpeta configurada del agente.

El pie del chat ocupa como máximo dos filas: cuenta, cuota y controles de acceso
en la primera; contexto, tokens y avisos en la segunda. Los textos largos se
abrevian visualmente y conservan su detalle en la ayuda emergente y al copiarlos.
La reanudación correcta no muestra el aviso «Conversación recuperada».

Cada chat muestra el usuario y la cuota restante cuando el proveedor los comunica.
Codex presenta las ventanas de **5 horas** y **semanal** en porcentaje, si están
incluidas en su respuesta, y la hora de renovación en la ayuda del indicador. Las
cuotas se consultan al conectar, al terminar un turno y cada minuto; también se
atienden las notificaciones del servidor. Pueden estar compartidas entre modelos.
Los agentes ACP y Ollama muestran **Contexto restante** si informan de sus tokens y
capacidad de contexto; es una métrica distinta de la cuota de la cuenta. Las API
muestran los tokens de la última petición. Los datos no proporcionados figuran como
**No disponible**. Los nombres de cuenta se muestran en la interfaz, sin guardarlos
en el perfil ni escribirlos en los registros.

**Ctrl+Intro** envía el mensaje y **Detener** cancela la respuesta del chat activo.
Cada respuesta de la IA, incluidas las recuperadas del historial, ofrece un icono
**Copiar respuesta** que conserva su texto Markdown original, listas y bloques de
código. Copia el texto actualizado que se muestra, también durante el streaming.
Ocultar la paleta conserva las conversaciones abiertas. El selector de agente superior
cambia el agente del chat activo; para conservarlo y trabajar con otro, utiliza **+**.

**Iniciar sesión** utiliza el acceso web del agente cuando está disponible:
Codex mediante su servidor oficial, Copilot mediante `login --web-flow` y Claude
mediante `claude auth login`. Otros adaptadores utilizan su método ACP anunciado.
El receptor local se mantiene mientras se autoriza en el navegador, aunque cambien
los permisos o se pulse de nuevo el botón de acceso. **Detener** cancela el acceso;
si se cancela o vence su espera de cinco minutos, hay que iniciar un acceso nuevo
desde el chat en lugar de recargar el callback anterior. Copilot abre el navegador
una sola vez; el chat conserva un enlace manual como alternativa.
Un método ACP que exige una terminal puede no permitir acceso web desde el chat;
se puede usar una clave API admitida por el proveedor.
Si el proveedor no ofrece ese flujo, configura su clave API y, para agentes nativos,
la variable de credencial correspondiente. El botón se oculta cuando Codex confirma
una sesión autenticada o Copilot tiene una cuenta guardada en su perfil. En Codex,
**Cerrar sesión** utiliza el cierre oficial del servidor y vuelve a mostrar el acceso.
El selector **Solo lectura / Pedir permiso / Acceso completo** está dentro del editor,
junto al modelo, y es común a todos los agentes y modelos. **Acceso completo**
autoriza las operaciones de las herramientas disponibles de ese chat sin nuevas
aprobaciones, incluidos comandos, cambios de archivos y las herramientas de
terminal habilitadas en el perfil. En Codex activa `danger-full-access` y
`approvalPolicy=never`; en ACP acepta las solicitudes de herramientas; las API
usan el mismo permiso para el puente de terminales y ejecución supervisada.

El permiso pertenece a ese chat y se mantiene al cambiar de modelo. Cada chat usa
un puente privado independiente. Para revocarlo, cambia el selector entre turnos;
no se deshacen operaciones ya ejecutadas. Los chats nuevos, reabiertos o recuperados
comienzan en **Solo lectura**: el acceso completo no se guarda en el historial ni
en los espacios. Las preguntas que requieren una respuesta del usuario siguen
mostrándose. No concede privilegios de administrador ni capacidades que el agente,
proveedor o sistema operativo no tengan.
Los controles de acceso son compactos; usuario y cierre de sesión comparten fila.
Los avisos rutinarios de agente listo y respuesta completada no ocupan una fila.

Las aprobaciones ofrecen **Permitir una vez**, **Permitir en esta sesión** y
**Permitir siempre** para la misma operación exacta y carpeta de ese agente.
Los cuatro botones de decisión se muestran en una sola fila compacta.
Los permisos de sesión del chat desaparecen al cerrar o cambiar su conexión;
los de terminal duran mientras está abierto ese chat y se limitan a esa
terminal y carpeta. Los persistentes se guardan como huellas, sin comandos ni
credenciales, en `permission-grants.json` dentro del directorio privado de cada
agente. Para revocarlos, elimina ese archivo con la aplicación cerrada.
Los permisos de terminal persistentes incluyen el destino y solo se ofrecen para
ejecutar cuando se conoce su carpeta de trabajo. Las aprobaciones del agente
permiten seguir usando la aplicación y se retiran al responder o al resolverse.

Cada perfil tiene su directorio privado bajo `$XDG_DATA_HOME/nexuterm/agents/`.
Su autenticación y extensiones no se comparten con otros perfiles ni se copian desde
la configuración global: los perfiles nuevos pueden requerir iniciar sesión otra vez.
Las claves introducidas en NexuTerm permanecen en memoria salvo que se guarden
explícitamente en el llavero del sistema. JSON solo conserva la referencia.
No introduzcas secretos en argumentos ni en el JSON de extensiones. HTTP solo está
permitido en loopback; el resto requiere HTTPS con validación TLS.

Con **Acceso a terminales** activado, los agentes compatibles reciben herramientas
para listar terminales abiertas, leer su pantalla y enviar comandos. Pueden elegir
la última terminal activa o una sesión concreta, local o SSH. Leer o ejecutar exige
aprobación gráfica con el agente, destino y operación, salvo que coincida con un
permiso recordado explícitamente o ese chat tenga **Acceso completo**. Los comandos se envían a la
sesión existente: la confirmación indica que se han enviado, no que hayan terminado
correctamente. Las API compatibles reciben estas herramientas como funciones;
los agentes nativos las reciben mediante un servidor MCP privado.

### Iniciar sesión en Codex con ChatGPT

En la paleta pulsa **Iniciar sesión web en Codex**. El chat consulta la cuenta
mediante `account/read` del servidor oficial. Si ya existe una sesión de ChatGPT,
la reutiliza sin abrir el navegador. En otro caso solicita el acceso web a Codex
y abre su URL en el navegador predeterminado; **Detener** cancela la operación.
Puedes escribir directamente en el chat después de autenticarte. **Ctrl+Intro**
envía el mensaje; las aprobaciones aparecen como diálogos con la operación que
se solicita autorizar. Cerrar un diálogo rechaza la operación.

Se utiliza el perfil seleccionado y su directorio privado. NexuTerm no lee ni
copia `auth.json` ni guarda tokens OAuth en `sessions.json`. Codex administra su
sesión mediante el `CODEX_HOME` exclusivo del perfil.

La cabecera reúne nueva conversación, historial, detener y ajustes. **Chats
recientes** consulta las conversaciones de la carpeta del perfil y permite
recuperarlas. En el editor inferior, **+** añade archivos o carpetas como rutas de
contexto; puedes quitarlas antes de enviar. Codex decide si necesita leerlas y
aplica sus permisos. El selector de modelos consulta `model/list`; el de
razonamiento muestra los niveles admitidos por el modelo. La elección se envía
con el siguiente mensaje. Los permisos se eligen antes del primer mensaje y
quedan fijados durante la conversación.

El micrófono ofrece dictado local con PipeWire (`pw-record`) y
[whisper.cpp](https://github.com/ggml-org/whisper.cpp). Este checkout tiene una
instalación opcional en `.local/whisper`: ejecutable `bin/whisper-cli` y modelo
multilingüe `models/ggml-tiny.bin` (unos 78 MB). NexuTerm la detecta automáticamente
cuando no has configurado otro motor; esta carpeta está excluida de Git y no se
elimina al limpiar `target`. No modifica la configuración global del equipo.

El idioma inicial es español. En **Configurar agentes de IA → General →
Configurar dictado local…** puedes elegir detección automática, otro ejecutable
o modelo. Si falta el motor, el micrófono abre ese mismo diálogo. La aplicación
no descarga modelos automáticamente. Pulsa el micrófono para grabar: termina automáticamente tras aproximadamente
1,5 segundos de silencio después de detectar voz. También puedes pulsarlo otra
vez para terminar; el máximo es un minuto. Las grabaciones sin señal suficiente
y las etiquetas de sonidos como `[Música]` no se añaden al borrador. El texto se añade al borrador para
revisarlo antes de enviar y el audio temporal se elimina al terminar. El audio
no se envía a una API. Tiny prioriza tamaño y recursos; puede cometer errores,
especialmente con ruido o frases muy breves.

El chat todavía no reproduce todas las funciones de VS Code: las solicitudes
de herramientas no admitidas se rechazan; no incluye los complementos, dibujo
ni objetivos que aparecen en el menú de esa extensión.

Si el navegador o su retorno local no funcionan, el CLI también ofrece
`codex login --device-auth`. Consulta la
[documentación oficial de autenticación](https://learn.chatgpt.com/docs/auth).

### MCP, skills, agentes y subagentes por perfil

La pestaña **MCP, skills y agentes** pertenece al agente editado. Sus botones añaden
plantillas JSON que se validan al guardar. Los servidores MCP se crean desactivados;
configúralos y actívalos explícitamente. Las skills referencian carpetas con `SKILL.md`;
NexuTerm no descarga paquetes. Los agentes incluyen nombre, descripción, instrucciones,
modo principal o subagente y modelo opcional.

OpenCode recibe su configuración nativa por proceso; Codex recibe `config.toml`,
skills y archivos de agentes en su perfil privado. Claude y Gemini reciben directorios
privados de skills y agentes. La ejecución de estas extensiones depende del adaptador;
las API de texto no ejecutan por sí mismas skills ni subagentes. No se escriben archivos
globales ni del proyecto. Inicia otra conversación para aplicar cambios de configuración.

Comprobaciones reproducibles, con datos ficticios y sin llamar a modelos reales:

```bash
cargo test --locked
python3 lab/check_ai.py
```

La comprobación GTK usa `xvfb-run` si existe o el `DISPLAY` actual y crea una sesión
D-Bus independiente. Guarda capturas y logs en `artifacts/`.

## Compilar y ejecutar

Desde el repositorio puedes recompilar con el script:

```bash
./recompilar.sh -r  # release
./recompilar.sh -d  # debug
```

Elige una sola opción. El script funciona también desde otro directorio, conserva
`Cargo.lock` y devuelve el error de Cargo si falla la compilación. Cargo debe estar
en `PATH`; si falta, se reutiliza una instalación existente en `~/.cargo/bin` o
en el entorno local de Rea, sin modificar la configuración global. También puedes
indicar su ejecutable mediante `CARGO=/ruta/a/cargo`. Se respeta `CARGO_TARGET_DIR`
si necesitas otra carpeta de compilación. Por defecto usa dos trabajos en paralelo;
puedes ajustarlos con `CARGO_BUILD_JOBS`.


Requisitos: Linux, Rust 1.92 o posterior, GTK >= 4.10, libadwaita >= 1.4 y VTE para GTK4 >= 0.70.
El objetivo nativo inicial es Ubuntu 24.04+ y Debian 13+; se ha comprobado el entorno indicado en `docs/VALIDACION.md`.
Para distros antiguas se necesitará un runtime empaquetado o backports compatibles: aún no se ha preparado Flatpak/AppImage.
El código no depende de apt durante la ejecución, pero no se afirma compatibilidad probada con todas las distribuciones.

Debian/Ubuntu (paquetes de desarrollo para compilar):

```bash
sudo apt update
sudo apt install build-essential pkg-config libglib2.0-dev-bin libgtk-4-dev libvte-2.91-gtk4-dev \
    libadwaita-1-dev libvncserver-dev libsecret-1-dev libssl-dev \
    openssh-client inetutils-telnet curl gvfs-backends dbus-user-session
```

RDP usa IronRDP compilado dentro del ejecutable y las bibliotecas TLS del sistema.
VNC usa directamente LibVNCClient; el paquete de release incluye su dependencia de biblioteca.
GTK/VTE y sus paquetes de desarrollo tienen nombres distintos en otras distribuciones.

```bash
cd ~/source/NexuTerm
~/.cargo/bin/cargo build --locked
~/.cargo/bin/cargo run --locked
```

Rust se instaló a nivel de usuario sin modificar el PATH del shell. Si `cargo` ya está en tu PATH, puedes usarlo directamente.
Para otras máquinas, instala Rust desde [rust-lang.org](https://rust-lang.org/tools/install/).

### Instalación en el escritorio Linux

La distribución recomendada para Debian/Ubuntu es un paquete `.deb` con sus
dependencias declaradas. Prepararlo requiere Python >= 3.11, `dpkg-dev` y `binutils`:

```bash
sudo apt install python3 dpkg-dev binutils
make release
sudo apt install ./dist/nexuterm_*.deb
```

`make release` compila la versión optimizada y genera el `.deb` en `dist/`.
APT instala OpenSSH, el cliente Telnet, el soporte FTP de GVfs y curl para subidas FTP y Windows CE, un proveedor de llavero (GNOME Keyring o KeePassXC), LibVNCClient y las bibliotecas necesarias, incluidas sus
dependencias transitivas. `dpkg-shlibdeps` calcula las bibliotecas y versiones
mínimas a partir del ejecutable; se respetan además los mínimos de GTK, VTE y libadwaita.
No uses únicamente `dpkg -i`: no descarga las dependencias que falten.

El paquete instala el programa en `/usr/bin`, junto con el lanzador y el logo.
Para empaquetar un binario ya compilado, usa `make package BINARY=target/release/nexuterm`.
El `.deb` necesita acceso a los repositorios de APT al instalarse; no es un paquete
autónomo sin conexión ni garantiza compatibilidad con distribuciones más antiguas.
Constrúyelo en la distribución más antigua que quieras soportar. Si APT no puede
satisfacer las versiones de las bibliotecas, rechazará la instalación.

`cargo build --release` y `make build` solo compilan: no instalan los clientes
externos. Para comprobar un equipo donde ejecutes el binario directamente:

```bash
make check-runtime
```

Alternativamente, `make build && sudo make install` instala en `/usr/local`,
pero comprueba antes los clientes SSH y Telnet, el backend FTP de GVfs y las bibliotecas enlazadas y falla
si falta alguno. No instala paquetes del sistema por su cuenta. RDP y VNC no requieren instalar visores externos.
El logo también queda integrado en el ejecutable y en **Acerca de NexuTerm**.
Para una instalación sin privilegios usa `make install PREFIX="$HOME/.local"`
(incluye `~/.local/bin` en tu PATH). `DESTDIR` permite preparar un árbol de instalación
sin comprobar los clientes del equipo constructor; sus dependencias deben resolverse
en el equipo de destino mediante el paquete `.deb`.
La instalación registra el lanzador en el menú; no copia accesos directos al escritorio personal.

## Uso

1. Abre **Conexiones** y pulsa **+** (o usa Ctrl+N) y selecciona SSH, RDP, VNC, Telnet, FTP o SFTP. El formulario muestra los campos del protocolo: usuario, clave y salto para SSH/SFTP; usuario y dominio para RDP; usuario y contraseña para FTP; servidor y puerto para VNC, con usuario opcional si el servidor lo requiere. Al cambiar de tipo se conserva el puerto escrito para cada protocolo. Grupo y favorito están en **Organización (opcional)**.
2. Guarda y pulsa **Conectar**. Telnet abre una terminal. FTP y SFTP abren un explorador gráfico local/remoto; selecciona archivos y utiliza **Subir** o **Descargar**. Las carpetas se abren con doble clic. SFTP solicita contraseña o frase de paso mediante una ventana gráfica cuando sea necesario. SSH abre una terminal; VNC abre el escritorio dentro de una pestaña y solicita la contraseña al pulsar **Conectar**. RDP también se abre dentro de una pestaña; introduce el usuario y la contraseña y pulsa **Conectar**. Para una CA privada, indica su archivo PEM en la pestaña; el certificado debe corresponder al nombre o IP del servidor.
3. Al abrir SSH aparece automáticamente **Archivos remotos**. Completa la autenticación y la comprobación de identidad en la terminal; el panel reutiliza esa conexión sin pedir otra contraseña. Doble clic abre una carpeta; también puedes escribir una ruta y pulsar Enter.
4. El icono **Subir archivo** permite elegir un archivo local. Para descargar, selecciona un archivo remoto y pulsa **Descargar**. Elige un nombre de destino que no exista: las transferencias normales no sobrescriben archivos existentes. En SSH/SFTP, el doble clic en un archivo regular abre su edición remota.
5. Copiar/pegar en terminal: **Ctrl+Shift+C / Ctrl+Shift+V**. El terminal interpreta los bytes pegados como cualquier shell interactiva.
6. Abre **☰ → Administrar sesiones** o pulsa **Ctrl+Mayús+S** para mantener los perfiles. Los iconos de carpeta y servidor de la cabecera alternan las paletas laterales de archivos y conexiones; en la paleta de conexiones están **+** y **Administrar sesiones**. Usa el menú principal para dividir la terminal activa, buscar (Ctrl+Shift+F), ajustar el zoom y abrir preferencias. Ctrl+Shift+T crea una terminal local; Ctrl+O abre el editor.

Si RDP presenta un certificado autofirmado, se muestra el servidor, el titular, la validez y la huella SHA-256 antes de autenticar. Comprueba la huella por otro medio con el administrador y elige **Confiar solo esta sesión** si coincide. **Cancelar** es la opción predeterminada. La confianza no se guarda: cada conexión sin una CA válida requiere confirmación. No se envía la contraseña hasta aceptar el certificado mostrado en esa misma conexión TLS.

El navegador utiliza **SFTP sobre SSH**. Terminal y archivos comparten un socket de control propio y privado. Si no hay conexión autenticada, el navegador espera o permite reintentar; no inicia una conexión independiente ni modifica `~/.ssh/config`.

Al seleccionar una pestaña SSH, **Archivos remotos** sustituye a las conexiones guardadas en el panel lateral. Al seleccionar otra clase de pestaña, vuelve el explorador local. F9 muestra u oculta el lateral. Cada pestaña SSH conserva su propio navegador y ruta. La navegación manual no sigue automáticamente los cambios de carpeta del shell.

### Edición remota SSH/SFTP

Doble clic en un archivo remoto regular descarga una copia a una carpeta temporal
privada (0700, archivo 0600) y abre el editor integrado. **Guardar** o **Ctrl+S**
escribe la copia local y utiliza SFTP sobre el socket OpenSSH autenticado del
terminal; no vuelve a pedir la contraseña ni modifica tu configuración SSH.
El nombre y destino originales se conservan y la pestaña identifica el servidor.
La opción de aplicaciones externas del explorador local no afecta a este editor.

Antes de subir se compara el original con la versión descargada. Se crea una
carpeta privada `.nexuterm-edit-<UUID>` junto al original y se escribe allí el
contenido por bloques de 64 KiB. Se verifica el temporal, se conservan y comprueban
propietario, grupo y permisos POSIX, y se vuelve a comparar el original antes de
publicar mediante `posix-rename@openssh.com`. El servidor debe admitir esta
extensión; si no la ofrece o no se pueden conservar los metadatos conocidos,
se bloquea el guardado. No hay alternativa silenciosa de escritura directa.
Se usa `fsync` del servidor si está disponible.

El reemplazo no trunca el original durante la subida. Tras publicarlo se verifica
su contenido y se actualiza la huella para el siguiente guardado. Se intenta
retirar el temporal; si una desconexión impide limpiarlo, el mensaje identifica
su ruta y se conserva la copia local. Si se pierde la confirmación durante el
reemplazo, el resultado se considera desconocido: comprueba el servidor antes de
reintentar. Las transferencias normales del explorador mantienen sus límites y
siguen rechazando sobrescrituras.

### Comparar y resolver versiones

**Comparar versiones…**, en la cabecera del editor, presenta el texto **Base**
(al abrir o tras el último guardado), **Local** (borrador sin guardar) y
**Original actual**, con las diferencias por líneas resaltadas. Un cuarto panel
permite editar el resultado. Puedes escoger toda una versión o componer el texto
manualmente; debes confirmar que lo has revisado antes de **Aplicar al borrador**.
No se mezclan cambios ni se sobrescribe el servidor automáticamente.

Aplicar vuelve a leer y comprobar el original comparado; si ha cambiado el texto,
propietario, grupo, permisos o fecha de modificación, exige una comparación nueva.
La aplicación solo actualiza el borrador y su base. Después pulsa **Guardar**, que
repite la comprobación de versión y la publicación atómica. Cancelar mantiene el
borrador y el original. Las comparaciones y la lectura SFTP se ejecutan fuera del
hilo gráfico. También se admite el editor GIO cuando el destino aporta etag.

Los borradores nuevos incluyen un campo opcional `base_text` en su JSON privado,
con hasta 8 MiB adicionales para el texto base; el texto editado conserva su límite
de 8 MiB. En SSH se valida que la base corresponda a la huella original guardada.
Los borradores anteriores siguen siendo legibles: si no incluyen la base, se
indica y se revisan las versiones disponibles sin inventarla. Para un borrador SSH
vincula antes una sesión del mismo perfil/destino; si existe un conflicto, el
vínculo queda limitado a revisión hasta aplicar una resolución explícita.

El contraste detallado se acota a 200.000 líneas por versión y dos millones de
celdas por pareja. Si se supera, se muestran los textos completos y se resaltan
regiones más amplias, con aviso. El resultado sigue siendo manual y no se trunca.
La comparación incluye los metadatos POSIX disponibles y el guardado los verifica;
no sustituye una revisión de ACL o atributos que SFTP no expone.

La operación sustituye el inodo y no actualiza otros enlaces duros al original.
SFTP v3 no permite garantizar la conservación de ACL, atributos extendidos u otros
metadatos no informados; la interfaz lo indica antes de guardar. Para archivos
que requieran esos metadatos, usa una herramienta que los preserve expresamente.
La comprobación de contenido no bloquea otros clientes: sigue existiendo una
ventana entre la última comprobación y el renombrado. No es un control de
concurrencia con bloqueo ni una garantía frente a fallos de energía del servidor.

El editor rechaza binarios, enlaces simbólicos, texto que no sea UTF-8 y archivos
mayores de 8 MiB. Los borradores se conservan para recuperar, comparar o guardar
una copia; descartar expresamente el documento retira también su borrador.

## Límites conocidos

- La ampliación profesional sigue incompleta: ejecución simultánea, gestor de túneles, panel de métricas, todavía no están conectados a la interfaz. Los módulos de backend en desarrollo no equivalen a funciones aceptadas.
- RDP y VNC se ajustan al área de la pestaña sin cambiar la resolución del servidor. No hay aún sincronización de portapapeles, audio ni configuración de certificados VNC personalizados. La compatibilidad con todos los servidores reales no está comprobada.
- La UI cubre conexiones básicas; no expone todas las opciones de gateway, multimonitor, impresión, smartcard, codecs o TLS de los protocolos de escritorio remoto.
- No se han implementado macros, diff, túneles configurables, Mosh, serie, servidores auxiliares ni importación de perfiles de otras aplicaciones.
- El explorador admite selección múltiple (hasta 128 elementos) y árboles de hasta 512 operaciones/32 niveles; no sigue enlaces ni acepta archivos especiales. La cola es compartida por destino, privada y persistente (512 entradas, 4 MiB de estado). Al recuperar una cola no se conecta ni transfiere hasta pulsar Iniciar. Pausar espera al archivo actual; reintentar comienza ese archivo desde cero. SFTP publica sin sobrescribir mediante temporales privados y enlaces exclusivos cuando el servidor los admite; sin soporte de enlace remoto, una subida interrumpida puede dejar un destino parcial. Las descargas locales usan temporal y publicación exclusiva. FTP comprueba previamente la ausencia del destino, pero el protocolo no garantiza exclusión frente a otros clientes y una interrupción puede dejar un archivo remoto parcial que debe revisarse. FTP Windows CE conserva transferencias individuales; los árboles no están soportados. SFTP usa bloques de 64 KiB y páginas de 200 entradas; los listados Windows CE se acotan a 8 MiB.
- El editor solo acepta UTF-8; no abre binarios ni archivos mayores de 8 MiB. En la edición local no guarda cuando el backend no ofrece etag; el guardado remoto SSH/SFTP utiliza la comprobación de contenido descrita arriba. El resaltado se procesa en segundo plano y está acotado al primer MiB y 65.536 segmentos de estilo; el resto sigue siendo editable como texto. Es resaltado sintáctico, sin análisis semántico ni servidor de lenguaje.
- Las contraseñas FTP se guardan opcionalmente con Secret Service (llavero del escritorio); el JSON solo contiene una referencia. Sin llavero disponible, desmarca **Guardar contraseña en el llavero** para conservarla únicamente durante esta ejecución. Un campo de contraseña vacío conserva la anterior si servidor, puerto y usuario no cambian. FTP entrega la contraseña directamente a GVfs; las subidas FTP y la compatibilidad con Windows CE utilizan curl sin abrir una consola y transmite la credencial por una tubería privada, sin archivos temporales ni argumentos del proceso. Las transferencias no sobrescriben archivos existentes. No se debilita la validación de certificados ni de host keys.
- Cerrar una terminal envía SIGHUP al grupo del proceso PTY y destruye el terminal; procesos deliberadamente desacoplados no quedan bajo supervisión de la app.
- El binario compilado en una distribución reciente no garantiza compatibilidad con glibc más antigua. Construir en la base más antigua soportada antes de distribuir.

## Validación

```bash
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
python3 -m unittest discover -s tests -p test_release.py
python3 lab/check_ui.py menus
python3 lab/check_local_explorer.py
python3 lab/check_browser_features.py
python3 lab/check_remote_editor.py
python3 lab/check_vnc.py
python3 lab/check_vnc.py --password
```

La prueba de menús requiere una compilación de desarrollo (`cargo build --locked`),
Xvfb y D-Bus. Repite la apertura y cierre de los diálogos desde el menú y comprueba
que el desplegable se oculta y que el logo está disponible con configuración aislada.

La prueba del explorador usa Xvfb y D-Bus con archivos y configuración temporales.
Comprueba el ancho del lateral al ampliar/reducir la ventana, navegación, numeración en claro/oscuro y apertura externa mediante una
aplicación MIME de prueba, incluyendo nombres con espacios y acentos. No cambia
las aplicaciones predeterminadas del usuario.

La prueba de edición remota crea un `sshd` desechable en loopback, claves propias
y una carpeta privada temporal bajo HOME (para respetar StrictModes); verifica SFTP
real, reemplazo de inodo, permisos, conflictos, interrupción durante la subida y
recuperación vinculada desde GTK. No usa servidores ni claves personales.
Requiere `sshd`, `ssh-keygen`, `ssh`, Xvfb y D-Bus.

Pruebas de biblioteca sin la interfaz GTK (el backend VNC requiere libvncclient):

```bash
cargo test --locked --no-default-features
```

Para una prueba breve de arranque con configuración aislada, consulta `docs/VALIDACION.md`.
Las pruebas de escritorio remoto necesitan servidores de laboratorio; no se usan equipos de producción.

## Documentación técnica

- [Arquitectura y dependencias](docs/analysis/README.md): organización del código, herramientas externas y alcance pendiente.
- [Aceptación operativa y matriz de distribuciones](docs/ACEPTACION.md): laboratorio reproducible y comprobaciones con operadores.
- [Historial de validación](docs/VALIDACION.md): resultados registrados por versión y comandos reproducibles; los resultados antiguos no certifican el checkout actual.
- [Pruebas de conexión de IA](docs/PRUEBAS_CONEXION_IA.md): observaciones fechadas, sujetas a la disponibilidad y autenticación de cada proveedor.

NexuTerm utiliza las APIs públicas y bibliotecas indicadas en `Cargo.toml`.

- [GTK Rust](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/)
- [VTE Rust](https://docs.rs/vte4/0.10.0/vte4/)
- [GIO](https://docs.gtk.org/gio/)
- [OpenSSH](https://www.openssh.com/manual.html)
- [IronRDP](https://github.com/Devolutions/IronRDP)
- [LibVNCServer/LibVNCClient](https://github.com/LibVNC/libvncserver)
