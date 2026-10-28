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

- Historial completo local de chats y resultados supervisados; reanudación API/ACP,
  adjuntos pendientes y estado visual del chat. La copia visible y la
  referencia nativa de Codex ya se integran en los espacios.
- Integración de pestañas FTP/SFTP y escritorios remotos, panel enfocado,
  estado visual restante y recuperación automática de estructura.
- Verificación específica de reconexión SSH restaurada contra el laboratorio;
  la prueba gráfica inicial de espacios utiliza terminales locales.

## Chats asociados al espacio

Implementado: copia privada de los mensajes visibles y el borrador, identidad,
orden, selección, modelo, carpeta y visibilidad de la paleta. Se restaura sin
conectar agentes ni transmitir contenido y se reutilizan los chats vivos sin
sustituir sus cambios. Un perfil eliminado conserva su copia consultable. Si se
supera el límite de ocho chats, se rechaza la apertura sin restauración parcial.

Codex conserva su referencia nativa y ofrece reanudación explícita en solo lectura;
no se permite enviar hasta confirmarla. Los fallos conservan la copia y el borrador.
API/ACP quedan como copias consultables, sin simular la continuidad del contexto.
Se mantiene la compatibilidad con espacios anteriores que no incluyen chats.

No incluye historial ilimitado (se conservan hasta los 128 mensajes visibles),
reanudación API/ACP, adjuntos pendientes, razonamiento, desplazamiento,
ni registro separado de resultados supervisados. El autoguardado de la copia
visible se describe en la entrega de recuperación automática.
`lab/check_workspace_chats.py` prueba persistencia y reapertura entre procesos,
reutilización, perfiles eliminados, capacidad, cierre y reanudación simulada de
Codex, incluidos errores, sin modelos ni servicios externos.

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

## Fases siguientes, en orden

1. **Transferencias:** selección múltiple, árboles, cola persistente, progreso,
   cancelación y reintento manual desde el inicio del archivo; temporales y
   publicación segura donde el backend lo permita, sin seguir enlaces ni
   sobrescribir por defecto.
2. **Editor remoto:** completar el diff de base/local/remoto y ampliar el
   tratamiento de metadatos donde el servidor lo permita. El temporal SFTP
   verificado y reemplazo atómico ya están implementados; la comprobación de
   conflictos no equivale a un bloqueo contra otros clientes.
3. **Ejecución supervisada:** procesos locales y canales SSH separados del shell
   interactivo; destino y directorio explícitos, aprobación, stdout/stderr,
   estado, duración y retorno opcional. Timeout inicial 60 s, captura hasta 4 MiB
   con indicación de truncamiento y resultado desconocido cuando corresponda.
4. **Recetas:** espacio en disco, estado de servicio y registros systemd; parámetros
   validados, comprobaciones finales y las mismas aprobaciones que la ejecución.
5. **Importación y túneles:** previsualización de perfiles OpenSSH sin ejecutar
   directivas; túneles locales/remotos/SOCKS supervisados, loopback por defecto y
   activación explícita.
6. **Intervenciones:** objetivo, destinos y cronología con evidencias; diagnóstico,
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
