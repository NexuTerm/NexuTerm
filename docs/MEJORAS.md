# Hoja de ruta del informe de mejoras

Acuerdo de trabajo: primero publicar los cambios pendientes y después comenzar
la implementación por fases. Se mantiene Rust/GTK, los backends existentes y la
autenticación y aprobaciones actuales, sin servicios externos nuevos.

## Entrega inicial: espacios de trabajo

Implementado: gestión de espacios con notas, árbol de divisiones de terminal,
orden y selección de pestaña, rutas locales y del explorador SSH, referencias a
perfiles y documentos del editor con identidad estable y borrador asociado. Persistencia privada, versionada y atómica; restauración desconectada
que añade pestañas y requiere una acción explícita para iniciar cada terminal.
Las divisiones SSH del mismo perfil comparten su conexión al restaurar.

Pruebas: persistencia y permisos; rechazo de datos inválidos y versiones
desconocidas; recorrido GTK en procesos separados para guardar, renombrar,
restaurar, iniciar expresamente y tratar carpetas ausentes o perfiles eliminados.
`python3 lab/check_workspaces.py` utiliza Xvfb, D-Bus y datos temporales.

Pendiente para cerrar la fase de espacios:

- Registro separado de resultados supervisados. La posición de lectura del chat
  ya se conserva y la llegada de respuestas respeta la lectura anterior.
  El historial local, las referencias reanudables y los adjuntos/razonamiento
  pendientes ya están integrados.
La recuperación de vistas básicas ya conserva cursor, selección y desplazamiento
por editor; ocultos, selección y desplazamiento por explorador; panel de terminal
activo y zoom; posición de lectura del chat. FTP/SFTP y RDP/VNC se recuperan
sin conexión. Las selecciones de explorador solo se aplican al listado cargado,
sin paginación adicional automática.

La reconexión SSH restaurada se ha validado contra el laboratorio propio con dos
divisiones y una conexión compartida. RDP restaurado se ha validado con imagen,
entrada, cierre y rechazo explícito de certificados no confiables.

## Recuperación automática de sesión

Implementado: captura periódica de la estructura compatible, documentos y copia
visible de los chats, sin depender de un espacio guardado manualmente. Conserva
primero los borradores, evita reescribir estados sin cambios y serializa las
escrituras por ventana en segundo plano. El cierre normal fuerza la última copia
y trata explícitamente los errores. Cada ventana utiliza su propia identidad y
bloqueo; las sesiones activas no aparecen como recuperables en otras ventanas.

Se ofrece recuperación al arrancar y desde el menú. Recuperar añade el trabajo
sin ejecutar terminales ni agentes, y solo retira la copia anterior después de
confirmar el reemplazo duradero. Los fallos permiten reintentar sin duplicar las
pestañas ya restauradas. El descarte es explícito y no borra originales ni
borradores. Se conservan los registros dañados y los espacios con nombre siguen
siendo independientes. Límite inicial: 32 copias de 16 MiB, sin expulsión automática.

Pendiente: los tipos de pestaña y estados visuales aún no compatibles. Una caída puede perder los cambios desde
la última captura completada; las escrituras o transferencias en curso posponen
la siguiente captura. No se vuelven a ejecutar operaciones al recuperar.

Prueba: `python3 lab/check_session_recovery.py`, con datos privados y SIGKILL real,
dos capturas sucesivas sin guardar un espacio, restauración al arrancar, fallo y
reintento de reemplazo, cierre normal, errores de almacenamiento, descarte y
registros dañados. Las pruebas de backend cubren bloqueo, separación entre
ventanas, permisos, retirada, límite de copias y preservación ante errores.

## Chats asociados al espacio

Implementado: copia privada de los mensajes visibles y el borrador, identidad,
orden, selección, modelo, carpeta y visibilidad de la paleta. Se restaura sin
conectar agentes ni transmitir contenido y se reutilizan los chats vivos sin
sustituir sus cambios. Un perfil eliminado conserva su copia consultable. Si se
supera el límite de ocho chats, se rechaza la apertura sin restauración parcial.

Codex conserva su referencia nativa. API recupera el contexto del último turno
completado sin peticiones al proveedor ni repetición de herramientas. ACP carga o
reanuda por referencia solo si el agente anuncia soporte, sin crear otra sesión ni
aceptar operaciones durante la reproducción. Se preservan las rutas de adjuntos
pendientes y el razonamiento anunciado por el modelo; los fallos permiten reintentar.

Implementado también: historial local separado de las 128 tarjetas visibles,
archivo incremental privado, consulta de conversaciones cerradas por páginas y
exportación JSONL completa. Límite global inicial de 1 GiB/1000 conversaciones,
cola acotada y aviso al detener la captura. Permite excluir el archivo completo
del chat y borrarlo expresamente, sin modificar las copias de recuperación ni
resucitar registros borrados. No contiene credenciales gestionadas ni aprobaciones.

Pendiente: otros estados visuales, registro separado de resultados
supervisados y validación contra proveedores reales. Las copias anteriores no
pueden reconstruir referencias o mensajes que nunca guardaron; los agentes ACP
sin capacidad de carga se mantienen consultables. Historial completo no significa
contexto ilimitado enviado al modelo ni almacenamiento sin límite.

Pruebas: `lab/check_workspace_chats.py` y `lab/check_chat_state.py`, con reanudación
API, carga ACP, fallo y reintento, adjuntos ausentes, razonamiento y 75 turnos que
mantienen 128 tarjetas visibles y 153 mensajes archivados. Se usan proveedores
sintéticos, sin servicios reales.

## Documentos asociados al espacio

Implementado: referencias a editores locales y SSH, conservación de borradores
antes de guardar y recuperación en el orden de las pestañas. La pestaña
seleccionada se mantiene; los editores ya abiertos se reutilizan sin cambiar su
posición ni su texto. El espacio conserva referencias al borrador actual, sin
copiar el contenido en su JSON ni congelar versiones.

Los originales locales limpios se leen de nuevo. Los borradores se recuperan sin
sobrescribir originales; los documentos SSH limpios requieren abrir expresamente
su original mediante una sesión existente. Si falta un borrador pendiente, se
avisa antes de ofrecer el original. Los registros dañados o de otro origen se
conservan; abrir el original en esos casos usa otra identidad para evitar que
su cierre elimine el registro anterior.

`lab/check_workspace_documents.py` verifica guardado seguido de SIGKILL,
reapertura, orden/selección, reutilización, cambios externos, originales ausentes,
borradores dañados y fallo de almacenamiento. El laboratorio SSH comprueba además
la apertura explícita de un original remoto limpio y su reutilización.

## Recuperación de documentos

Implementado: borradores privados, versionados y atómicos del texto modificado,
una escritura por editor en segundo plano, captura periódica y conservación de
la última edición antes del cierre normal. Lista al arrancar y desde el menú;
recuperar, comparar con el original y exportar una copia nueva sin sobrescribir.
El guardado local mantiene el etag de origen; conflictos y originales ausentes
bloquean la sobrescritura. Guardar correctamente o descartar retira el borrador.
Los borradores remotos son independientes de la copia temporal y se recuperan
sin conexión. Ya pueden vincularse expresamente a una terminal autenticada del
mismo perfil/destino si el original coincide con su huella SHA-256. El texto se
conserva, el guardado vuelve a validar la versión y la siguiente edición recuerda
la nueva huella. Los borradores anteriores sin huella siguen ofreciendo copia y
comparación, sin subirlos automáticamente.

Pruebas: `lab/check_recovery.py` fuerza una caída con SIGKILL y valida reapertura,
guardado, comparación ante conflicto, original ausente, recuperación remota sin
conexión, descarte y cierre normal con fallo de almacenamiento. La integración
SSH del editor verifica que el borrador persiste al eliminar su copia temporal,
la vinculación desde GTK y el rechazo de cambios externos y destinos incorrectos.

Implementado también como requisito del guardado recuperado: temporal SFTP privado,
verificación y reemplazo atómico con preservación de propietario/grupo/permisos
POSIX. No hay alternativa de escritura directa. El reemplazo cambia el inodo y
no garantiza ACL ni atributos extendidos; la interfaz y el README lo indican.

## Transferencias y recientes por agente — 30 de octubre de 2026

Implementada la selección múltiple y los árboles FTP/SFTP desde el explorador
local principal, mediante portapapeles y arrastre en ambos sentidos. La cola por
destino conserva estados, intentos y errores, permite pausa/cancelación y reintento
manual desde el inicio del archivo. Recuperar no conecta ni ejecuta pendientes.
Las carpetas se planifican antes de escribir y cada operación terminada se guarda.
Los límites y las diferencias de publicación FTP/SFTP figuran en el README.

Chats recientes pasa a ser una vista completa alternativa a la conversación,
con editor visible y listado por perfil de agente. Elegir un chat o empezar a
escribir sustituye esa vista en la misma pestaña. El retorno guarda la conversación
anterior; los recientes y los JSONL comparten cuota y borrado sin resurrección.

## Editor remoto: comparación y resolución — 30 de octubre de 2026

Implementada la revisión Base/Local/Original actual, diferencias por líneas y
resultado editable con confirmación explícita. Aplicar conserva el resultado como
borrador y verifica de nuevo el original; no publica cambios. Guardar mantiene la
verificación independiente y el reemplazo atómico SFTP. Un cambio de contenido,
permisos, propietario/grupo o mtime invalida la versión que se revisó.

Se conserva la base en los borradores nuevos mediante un campo opcional compatible
con los anteriores. Un borrador SSH cuyo original cambió puede vincularse para
revisión, con guardado bloqueado hasta confirmar la resolución. Los antiguos sin
base se muestran como tales. Los metadatos disponibles se presentan y se conservan
los permisos POSIX, incluidos bits especiales; ACL/xattrs siguen fuera de las
capacidades del transporte actual. Se mantienen los límites de edición y se acota
el coste del contraste, sin añadir dependencias ni hacer fusiones automáticas.

## Ejecución supervisada — 30 de octubre de 2026

Implementados procesos locales con grupo propio y canales SSH separados de la
terminal interactiva, ligados a destino/carpeta/comando aprobados. Hay interfaz
propia y herramientas MCP, aprobación nueva por ejecución (salvo acceso completo
autorizado para ese chat), stdout/stderr, duración,
retorno opcional, timeout inicial 60 s y captura total de 4 MiB con aviso de
truncamiento. Se distingue resultado desconocido en SSH sin afirmar terminación
remota cuando se cancela o pierde el canal.

El historial privado conserva capturas periódicas/finales, permite exclusión,
consulta, exportación y borrado confirmado, con cuota de 1 GiB/1000 archivos y
reserva previa. Una captura recuperada es evidencia; no reanuda ni repite comandos.
El cierre cancela y espera los activos, protege salidas sin historial confirmado
y permite exportar o descartar expresamente. Se acotan concurrencia, memoria y
lecturas de historial; no se añaden dependencias ni se altera la entrada del shell.

## Acceso completo por chat — 30 de octubre de 2026

El selector común añade Acceso completo para Codex, agentes ACP y API. Autoriza
las herramientas disponibles sin aprobaciones repetidas y se conserva al cambiar
de modelo. Codex recibe la política sin sandbox y sin preguntas de aprobación;
ACP acepta sus solicitudes y el puente privado aplica el permiso a terminales y
procesos supervisados. Cada chat tiene su propio socket; el permiso no se propaga
a otros chats ni se guarda al cerrar, recuperar o reabrir una conversación.
Se puede revocar entre turnos. Las preguntas al usuario siguen siendo manuales,
y se mantienen validación de argumentos, cuotas y límites de ejecución.

## Fases siguientes, en orden

1. **Recetas:** espacio en disco, estado de servicio y registros systemd; parámetros
   validados, comprobaciones finales y las mismas aprobaciones que la ejecución.
2. **Importación y túneles:** previsualización de perfiles OpenSSH sin ejecutar
   directivas; túneles locales/remotos/SOCKS supervisados, loopback por defecto y
   activación explícita.
3. **Intervenciones:** objetivo, destinos y cronología con evidencias; diagnóstico,
   propuesta, aprobación, cambio y verificación; recuperación sin repetir acciones
   y exportación revisable Markdown/JSON con exclusión de contenido sensible.

## Reglas de aceptación

- Cada entrega incluye backend, interfaz, documentación y pruebas pertinentes.
- Conservar historial completo local por defecto, sin credenciales; permitir
  borrado y exclusión por ejecución. Aplicar al implementar el límite inicial de
  1 GiB a las salidas; al llenarse, avisar y detener nuevas capturas, sin borrar
  documentos pendientes.
- Restaurar nunca ejecutará comandos, recetas, colas ni túneles automáticamente.
- Validar transferencias interrumpidas y recuperación con servidores desechables;
  no usar equipos industriales o servicios de producción.
- Medir preparación de entorno, copia de carpeta, recuperación y diagnóstico;
  realizar validación de uso con soporte/administradores antes de cerrar el modo
  de intervención.
- Mantener paquete Debian; validar en Ubuntu 24.04 y Debian 13 antes de anunciar
  compatibilidad. Flatpak/AppImage queda fuera de esta hoja de ruta.
