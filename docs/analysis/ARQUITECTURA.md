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
`Resumed`; la copia local permanece si el proveedor falla. API/ACP no reanudan
contexto en esta entrega. No se serializan concesiones ni peticiones de aprobación.

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
