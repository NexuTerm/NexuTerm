# Validación de NexuTerm

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

## Entorno utilizado

- Ubuntu 26.04.1 LTS en WSL, x86_64.
- Rust/Cargo 1.99.1, instalados a nivel de usuario con rustup y sin modificar el PATH.
- GTK 4.22.4; VTE GTK4 0.84.0.
- Disponibles OpenSSH, `xfreerdp3`, TigerVNC `vncviewer`, GVfs, Xvfb y D-Bus.

## Resultados

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
| Instalación MobaXterm | SHA-256 de los seis originales coincide con el inventario inicial |

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

## Pendiente de aceptación

- Login SSH por contraseña/clave/agente y salto contra servidores de laboratorio.
- Navegación, autenticación, transferencias, cancelación y edición SFTP reales.
- Conexiones y desconexiones RDP/VNC, teclas, portapapeles y resize reales.
- Cierre de sesiones interactivas y comportamiento de los clientes frente a fallos de red.
- Pruebas en Ubuntu 24.04, Debian 13 y otras distribuciones; validación Wayland nativa.
- Empaquetado redistribuible para runtimes antiguos; Flatpak/AppImage aún no preparados.

No se ha inicializado Git, hecho commit, publicado ni desplegado el proyecto.
El repositorio HMI no se modificó. Los artefactos de compilación quedan en `target/`,
excluido por `.gitignore`.
