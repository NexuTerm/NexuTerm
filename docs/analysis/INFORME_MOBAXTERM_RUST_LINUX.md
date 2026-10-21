# MobaXterm: análisis de la instalación y especificación para Rust/Linux

Fecha: 21 de octubre de 2026. Instalación examinada: `C:\Program Files (x86)\Mobatek\MobaXterm`, accesible en WSL como `/mnt/c/Program Files (x86)/Mobatek/MobaXterm`.

## Resultado y alcance

Esta instalación combina una aplicación gráfica principal (`MobaXterm.exe`), un ejecutable auxiliar multimodo (`MobaRTE.exe`) y un entorno de utilidades empaquetadas para Windows/Cygwin. La réplica funcional para Linux debe tratar por separado la interfaz y coordinación de sesiones, los protocolos remotos, el editor/comparador y las herramientas Unix.

El inventario de archivos y contenedores es exhaustivo dentro de la carpeta indicada. El análisis funcional es estático y tiene cobertura desigual: cadenas, recursos, cabeceras PE, dependencias y fragmentos de código permiten establecer capacidades y rutas concretas, pero no demostrar **todo** el comportamiento de todos los programas. No se ejecutaron los EXE de MobaXterm, no se abrieron sesiones remotas y no se consultaron credenciales ni perfiles personales.

Este documento es una base de especificación y planificación; no certifica equivalencia de una implementación futura. Cada función debe convertirse en un contrato observable y validarse mediante las pruebas propuestas.

## Lectura del expediente

- Este informe: visión de conjunto, ejecutables principales y límites de cobertura.
- `CATALOGO_EJECUTABLES.md`: catálogo de utilidades internas por ruta y tipo real.
- `PROPUESTA_RUST_LINUX.md`: arquitectura, tecnologías candidatas, fases y aceptación.
- `inventario-raiz.json`: tamaño y SHA-256 de cada archivo original de la carpeta.
- `plugins-inventory.json`: entradas ZIP, identidades y análisis de contenido derivado.
- `rea-evidence-inicial.json` y posteriores exportaciones REA: evidencias estructuradas con identificadores y localizaciones.
- `MobaXterm.exe.pe.txt`, `MobaRTE.exe.pe.txt`: cabeceras e importaciones observadas mediante GNU objdump.
- `*.strings.json`: cadenas ASCII/UTF-16LE y offsets de archivo; un literal aislado es un indicio, no una prueba de ejecución.

## Método y niveles de certeza

1. Inventario recursivo y hashes SHA-256 de originales.
2. Clasificación de formatos por contenido, sin confiar en las extensiones.
3. REA `inspect_pe_resources` para árboles completos de recursos de los dos EXE principales.
4. REA con Ghidra para consultas de código nativo, cuando la importación y el autoanálisis terminan.
5. Inspección de ZIPs y descompresión estática de contenidos internos, conservando por separado identidad original y derivada.
6. Contraste con documentación primaria del producto y de componentes propuestos para Linux.

**Observado** significa presente en bytes o en resultados de herramientas. **Inferido** significa interpretación de esos indicios. **Documentado** significa declarado por el proveedor. **Pendiente** significa que falta prueba estática concluyente o una validación dinámica. Las APIs importadas y clases compiladas no garantizan que todas sus capacidades sean alcanzables desde esta edición.

## Archivos de primer nivel

| Archivo | Tamaño | Tipo observado | Papel |
|---|---:|---|---|
| `MobaXterm.exe` | 16.474.416 B | PE32 x86 GUI, 8 secciones | Aplicación principal y coordinador de sesiones, recursos y herramientas |
| `MobaRTE.exe` | 6.426.928 B | PE32 x86 GUI, 8 secciones | Auxiliar gráfico con múltiples modos de trabajo |
| `CygUtils.plugin` | 19.476.701 B | ZIP, 1.050 entradas | Entorno/utilidades predominantemente de 32 bits; arquitectura real por contenido |
| `CygUtils64.plugin` | 13.175.720 B | ZIP, 394 entradas | Entorno/utilidades predominantemente de 64 bits; arquitectura real por contenido |
| `MobaXterm_installer.dat` | 32.652.957 B | ZIP con dos entradas | Contenedor de los dos plugins; comparar hashes antes de tratarlo como otra versión |
| `version.dat` | 15 B | Texto ASCII/CRLF | Contiene las líneas `26.4`, `5512`, `0`; los dos últimos valores no se interpretan como contrato sin más evidencia |

Hashes principales:

```text
MobaXterm.exe c581f9313c56c4b848d652420a04e3fb45cebbe3111ce02bb8a92343f01d87e4
MobaRTE.exe   5a24eb19237092e6fc5ea129e0916df8667cdd424edc1398614e467368bbf15d
```

Se contrastó VERSIONINFO por sus rangos de recursos: `MobaXterm.exe` declara `FileVersion=26.4.0.5512` y `ProductVersion=26.4`; `MobaRTE.exe` declara `FileVersion=1.8.68.0` y `ProductVersion=1.8`. La versión principal concuerda con `version.dat`. Son metadatos observados, no una verificación de firma/autenticidad del proveedor.

Se verificó que ambos plugins contenidos en `MobaXterm_installer.dat` tienen el mismo SHA-256 que los archivos externos. En cambio, el `MobaRTE.exe` recuperado de MOBASE **no** tiene el mismo SHA-256 que el ejecutable de primer nivel: el análisis Ghidra de MobaRTE de este informe corresponde exclusivamente al de primer nivel.

## MobaXterm.exe: aplicación principal

### Implementación y dependencias observadas

Binario nativo Windows de 32 bits. Las cadenas `TMobaXtermForm`, referencias a unidades Pascal y componentes VCL son consistentes con Delphi; no equivalen a disponer de su fuente. Las importaciones observadas incluyen `kernel32`, `user32`, `advapi32`, `gdi32`, `ole32`, `oleaut32`, `comctl32`, `imm32`, `shell32`, `comdlg32`, `ws2_32`, `gdiplus` y `Crypt32`. Hay nombres duplicados de DLL en la tabla: no representan instancias independientes del programa.

Las dependencias de ventanas, COM/ActiveX y registro de Windows se deberán sustituir por servicios propios de Linux. La presencia de `ws2_32` confirma enlaces a Winsock; no enumera por sí misma los protocolos realmente ejecutados.

### Capacidades, evidencia y requisito de réplica

| Área | Evidencia estática local | Interpretación y requisito en Linux |
|---|---|---|
| Interfaz de sesiones y pestañas | `TMobaXtermForm`, `-newtab` en offset `0x591520`, `-bookmark`, `-session`, `-bookmanager` | Gestor de sesiones persistentes, pestañas, restauración y cierre con supervisión; gramática completa pendiente |
| Terminal local | Referencias a plugins Cygwin y utilidades, `-hideterm`, `-exitwhendone` | Un PTY por sesión local, tamaño, entrada/salida incremental, señales y cierre limpio |
| SSH y autenticación | Nombres de algoritmos SSH, `TElOpenSSHAuthAgentS`, `MobAgent`, formatos PuTTY/OpenSSH | Cliente SSH y agente; validar host keys, autenticación y compatibilidad por negociación real |
| Túneles SSH | `MOBATUNNEL:`, mensajes de inicio del subproceso y `-realhost` | Reenvíos locales/remotos/dinámicos y jump hosts; los mensajes no prueban todos los modos |
| SFTP/SCP y edición remota | `MobaSFTP`, `-Upload`, `-Download`, `-OpenFile`, `/usr/lib/openssh/sftp-server` | Navegador de archivos remoto, transferencias cancelables y edición con control de conflictos |
| FTP/FTPS | Opciones `-passive`, `-UseSSL`, `-EncryptDataChannel`, `-implicitSSL`, variantes TLS | Módulo separado con certificados y modos activo/pasivo; matriz de interoperabilidad pendiente |
| RDP | `mobardp`, `TMobaRDPForm`, referencia al auxiliar | Coordinación con cliente RDP; detalles de implementación en MobaRTE |
| VNC | `-viewonly`, `-autoscale`, `-donotrespectratio` y propiedades de proxy | Visor remoto con escalado/entrada/portapapeles, sujeto a confirmación de ruta de llamada |
| X11/XDMCP | Recurso `XW`, `-xdmcp`, distribución de bibliotecas X | Integración con X11/Xwayland en Linux; no portar sin más la adaptación Windows |
| Herramientas y paquetes | `TFormMobApt`, `-mobapt`, `-mobapt64`, creación de plugins | Resolver utilidades y dependencias con paquetes Linux y control explícito de instalación |
| Editor/diff/visor | `MobaTextEditor`, `-gitedit`, `-compfiles`, `-compfolders`, `-picview` | Delegación de documentos y comparaciones; evitar duplicar el editor en cada módulo |
| Macros | `-runmacro` en `0x5bc488` | Grabar/reproducir acciones con un formato propio validado; no prometer compatibilidad binaria |
| Monitorización remota | `MobaRemoteMonitoring`, `-remotemonitoring` | Sondeos acotados, cancelables y aislados por sesión; comandos exactos pendientes |
| Configuración y credenciales | `MobaXterm.ini`, rutas `Software\\Mobatek\\MobaXterm`, texto de contraseña maestra | Persistencia propia y almacén de secretos Linux; no se inspeccionó ninguna contraseña real |
| Integración Windows | `-installprotohandler`, `-installshellext`, manejo de ventanas | Sustituir por `.desktop`, esquemas URI y servicios del escritorio; no requiere imitación del registro |
| Instalación/despliegue | `-extractdeps`, `-justunpack`, `-instunpack` | Empaquetado Linux separado de la ejecución normal; opciones observadas no se ejecutaron |

Los offsets pertenecen al archivo original identificado por SHA-256; no son direcciones virtuales. Las opciones internas encontradas en cadenas son candidatos a investigar, no una CLI pública validada. No se deben copiar opciones que desactivan validaciones de seguridad como valores predeterminados.

### Recursos incorporados

REA examinó **451 entradas y 220 recursos**, con cobertura completa dentro de su parser acotado. Evidencia: `ev_f19eda0aefb0c12ae0c1beff364864a7d93665022451ae4d9eef2e9b9e8625b7`.

El tipo de recurso llamado `EXEFILE` incluye `BASEPROF`, `CHANGELOG`, `CUSTGLYPHS`, `FLICO16/22/32/48/64`, `HOTKEYS`, `ICONSOLD`, `IMGEXTS`, `IMPINGUI`, `IMTHEMES`, `LI`, `MOBAFANTA`, `MOBAFIRA`, `MOBAFONT`, `MOBASE`, `SEPARAMS`, `SKINS`, `TXTRES` y `XW`. **El nombre EXEFILE no demuestra que cada payload sea un EXE.** Es necesario clasificar/descomprimir cada uno.

Dos recursos que merecen atención por su tamaño son `MOBASE` (offset `8909080`, 2.503.485 bytes) y `XW` (offset `11596408`, 4.140.818 bytes). Los nombres sugieren contenido base y X, pero la función debe confirmarse mediante el contenido derivado.

## MobaRTE.exe: auxiliar gráfico multimodo

### Responsabilidad

No debe reducirse a «editor de texto». Sus cadenas contienen varios formularios y un bloque de selección de modos. La hipótesis más fuerte es que actúa como ejecutable auxiliar gráfico compartido para editor, diferencias, navegadores, pestañas de programas externos y RDP. La presencia de varias generaciones de formulario RDP no prueba que todas se utilicen en el mismo Windows.

### Modos y comportamiento a reproducir

| Modo | Evidencia local con offset | Función inferida | Diseño para Linux |
|---|---|---|---|
| Editor | `-moteed` `0x335454`, `MobaTextEditor` `0x335464`, clases SynEdit/highlighters | Edición de documentos, resaltado y manejo de archivos locales/remotos | Editor con Unicode, búsqueda/reemplazo, codificación/EOL, escritura atómica y conflictos remotos |
| Comparación de archivos | `-compfiles` `0x33560c`, `MobaDiff` | Comparación visual de dos archivos | Diff por líneas y navegación de cambios; formatos binarios deben declararse aparte |
| Comparación de carpetas | `-compfolders` `0x335620` | Recorrido/comparación de árboles | Recorrido incremental, cancelación, política de symlinks, tamaños/fechas/hash opcional |
| Diff contextual | `-contextdiff` `0x33565c`, `-mobadiff` `0x335638` | Entrada adicional al comparador | Definir formato de salida propio; verificar contexto y finales de línea con corpus |
| Búsqueda en carpetas | `-searchindir` `0x3356c4`, `-casei`, `-incbin` | Búsqueda multiarchivo y tratamiento de mayúsculas/binarios | Resultados paginados con límite y cancelación, evitando cargar árboles completos |
| Vista rápida | `-quickshow` `0x33547c`, `MobaQuickShow` | Presentación rápida de contenido; formatos exactos sin confirmar | Selección de visor por formato; verificar imágenes/texto y límites de tamaño |
| Navegador COM/IE | `-browser` `0x3354d4`, `TFormMobaBrowser`, `-emulateie7`…`-emulateie11` | Navegación embebida mediante componentes Windows heredados | WebKitGTK o navegador externo; no reproducir modos IE |
| Navegador externo | `-firefox`, `-chrome`, `TFormMobaExtBrowser`, `-browserexe` | Lanzar e integrar navegador externo | Apertura externa como mínimo; incrustación portable bajo Wayland es una función distinta |
| Navegador Edge | `TFormMobaEdgeBrowser`, nombres WebView2, `-userprofile` | Navegador basado en WebView2 y perfiles | WebKitGTK/perfil propio o navegador externo; no depender de Edge/COM |
| Contenedor de ventanas | `-tabber` `0x335560`, `MobaTabber`, `-execfile`, `-execparams`, `-timeout` | Ejecutar programa externo y coordinar su ventana | Supervisión de procesos; no garantizar reparentado arbitrario en Wayland |
| RDP antiguo/actual | `TFormMobaRDP1/2/3`, `TMobaRDPForm`, propiedades de conexión | Envolver controles RDP de Windows con distintas capacidades | FreeRDP o implementación RDP Rust evaluada por prestaciones concretas |
| Coordinación con principal | `-mobahwnd`, `-numtab`, mensajes de elevar ventana | Identidad de ventana/pestaña y comunicación con la aplicación principal | IDs de sesión y canales IPC tipados; no trasladar HWND al diseño Linux |
| Descarga | `-download` `0x335440`, `MobaWget.log` | Modo de descarga auxiliar; alcance aún por confirmar | Transferencia separada con progreso, cancelación y validación TLS |

### Superficie RDP observada

El bloque de cadenas de RDP contiene opciones de resolución y escalado (`-displayres`, `-autoscale`, `-zoomlevel`, `-autoAdaptResolution`, `-multimon`), teclado (`-forwardkeys`), color (`-colordepth`), gateway (`-rdpgateway`, `-autodetectgw`, `-rdgwlogin`, `-rdgwdomain`, `-rdgwsharedcreds`), autenticación (`-smartcard`, `-authlevel`, `-domain`), portapapeles (`-RedirectClipboard`), audio/micrófono (`-audiomode`, `-redirectMicro`), discos/impresoras/puertos y presentación (`-displaywallpaper`, `-displaythemes`, `-smoothfonts`, `-desktopcomp`).

Para Rust/Linux, cada opción es un requisito candidato que se debe mapear al backend seleccionado y probar. Encontrar un literal no demuestra soporte efectivo, valores permitidos ni semántica idéntica. No se han probado gateways, tarjetas inteligentes, impresión, DirectX ni multimonitor.

### Recursos e implementación

REA examinó **235 entradas y 112 recursos**, con cobertura completa dentro de su parser acotado. Evidencia: `ev_0bff6cb6e42d22020e51788dc39686e4bde22228ee11e23e8979b9dd9ca157ce`.

Recursos `EXEFILE`: `MOBAFANTA`, `MOBAFIRA` y `MOBAFONT`. Su contenido se clasificó como fuentes TrueType, no ejecutables. Las importaciones observadas son principalmente Win32, GDI, COM/OLE, controles, shell e IME. La carga de componentes mediante COM o resolución dinámica puede no aparecer como DLL explícita.

## Paquetes Cygwin y catálogo de programas

Los plugins son archivos ZIP, pero parte de sus entradas contiene una segunda capa de compresión o enlaces Cygwin. Hay 268 nombres con extensión `.exe` en `CygUtils.plugin` y 194 en `CygUtils64.plugin`; **462 nombres no equivalen a 462 implementaciones independientes**.

Se deben distinguir:

- PE nativo real tras descompresión.
- Script con nombre `.exe` que delega en otro programa.
- Enlace Cygwin que selecciona un applet de BusyBox/MobaBox o un subcomando Git.
- Binario compartido bajo varios nombres con SHA-256 idéntico.
- Contenido opaco o no reconocido, cuya función sigue pendiente.
- Contenido dentro de `.mobapack`, que no debe describirse como desplegado sin observar su extracción.

El anexo de catálogo conserva por programa su ruta, clasificación, destino cuando sea enlace, hashes y papel funcional. No se debe decidir una reescritura en Rust antes de resolver esas delegaciones. En Linux, muchas funciones ya existen como utilidades nativas; una aplicación Rust puede coordinarlas sin reimplementar sus algoritmos.

### Resultado de descompresión y cobertura interna

Además de los plugins externos se recuperaron dos ZIP derivados de los recursos `MOBASE` y `XW`. El inventario conserva 974 archivos regulares de CygUtils, 356 de CygUtils64, 42 de MOBASE y 114 de XW; los recuentos iniciales de entradas ZIP incluyen también directorios. Se examinaron semánticamente **675 candidatos** de bin, EXE, DLL y mobapack: 303 PE reconocidos, 286 enlaces Cygwin, 60 scripts, 17 opacos y 9 MZ cuyo PE no pudo analizarse con el parser local. Entre ellos hay **485 ocurrencias con sufijo .exe, correspondientes a 282 nombres distintos**. No se cuenta dos veces el instalador que contiene copias de los plugins.

Para algunos contenidos LZMA se normalizó a cero el byte 1 de una copia de su cabecera antes de usar 7z; solo se aceptó una salida con tamaño igual al declarado. Esta transformación es una hipótesis de decodificación documentada, no un cambio en la instalación original. Los hashes de origen y resultado permiten reproducir y revisar el paso.

Se observaron **53 discrepancias CRC32** en los candidatos de MOBASE/XW. Sus miembros se conservaron como bytes observados no confiables, sin ejecutarlos. La causa no está establecida: puede relacionarse con personalización/empaquetado y no prueba por sí misma malware. No se afirma integridad ZIP completa de esos contenedores. Las conclusiones sobre esos derivados tienen esta limitación adicional.

### Auxiliares específicos de MobaXterm encontrados dentro de MOBASE

| Ejecutable | Evidencia de contenido | Función candidata y réplica |
|---|---|---|
| `MoTTY.exe` | PE32; cadenas de integración con `TMobaXtermForm` y manejo de terminal | Cliente de terminal integrado, con indicios de familia PuTTY. En Linux separar motor terminal, transporte SSH y coordinación GUI; no asumir misma semántica de argumentos |
| `MobAgent.exe` | PE32; texto sobre configuración OpenSSH y formato de claves PuTTY | Agente de autenticación SSH. Integrar el agente del usuario; protocolo IPC y acciones exactas pendientes |
| `MobaKeyGen.exe` | PE32; referencias explícitas PuTTYgen y exportación de clave pública OpenSSH | Generación/importación/conversión de claves; definir formatos admitidos y comprobarlos con herramientas independientes |
| `MobaSCP.exe` | PE32; marcador `##END_Of_MobaSCP_Request##`, canal de archivos abiertos y comandos de sondeo | Auxiliar de transferencias y comunicación con el principal; protocolos y gramática interna pendientes |
| `MobaRTE.exe` | PE32 con formularios editor/RDP | Copia interna del auxiliar: comparar hash con el EXE externo antes de dar por idénticas ambas versiones |
| `c32termd.exe`, `c64termd.exe`, `m32termd.exe`, `m64termd.exe` | PE de arquitecturas x86/x86_64; imports y cadenas en catálogo | Adaptadores de terminal Windows; reemplazar por PTY Linux y supervisión Rust, sin copiar el IPC aún desconocido |
| `stgames.exe` | PE32 con referencias a formato de partidas | Juego/colección auxiliar; no es requisito del núcleo de acceso remoto |

MOBASE contiene además helpers sin sufijo EXE (`wsltermd`, `arm_wsltermd`, `wslconnect`, `wineconnect`, `mac_wineconnect`). Su presencia requiere clasificación por bytes y arquitectura, no por extensión. En esta pasada algunos quedan opacos; no se afirma recuperación funcional de estos helpers. XW incluye gestores de ventanas y utilidades X, detallados en el catálogo. La presencia de `OpenX.exe` no permite identificarlo como servidor X completo solo por su nombre.

### Comprobación de código de MobaRTE con REA/Ghidra

El autoanálisis de Ghidra 12.1.4 terminó e inventarió **9.591 procedimientos y 17.811 cadenas definidas**. Evidencia `ev_95951ca9b0fa756c6345fc75e97214f5669dfb38c7cc647d6e65faff1f2f9cca`. Esos recuentos cubren la base de análisis, no demuestran que se hayan reconocido todas las funciones ni todos los literales.

La consulta `procedure_pseudo_code` del selector `file-offset:0x334000` resolvió la función `entry`; evidencia `ev_8363e1d784a2f08d01ed56d2a080cda6214996de0113eadb1ffd0e93a6bb1f2c`. El pseudocódigo confirma ramas de selección para editor, vista rápida, navegador, Firefox/Chrome/Edge, contenedor de ventanas, RDP y diff/búsqueda. También muestra `CreateSemaphoreA`, `FindWindowA` y `SendMessageA(..., 0x4a, ...)`, consistentes con instancia única y mensajes WM_COPYDATA para remitir archivos a una ventana existente.

Esto eleva el despacho multimodo desde indicios de cadenas a una observación de flujo estático. No demuestra ejecución de esas ramas ni recupera la semántica completa de cada formulario. La descompilación de Delphi muestra tipos y variables inciertos, por lo que no debe traducirse literalmente a Rust.

### Comprobación de código de MobaXterm con REA/Ghidra

El autoanálisis del EXE principal terminó: **18.276 procedimientos y 22.784 cadenas definidas** en Ghidra 12.1.4; evidencia `ev_db16c822e3cc112e70aacce8a9b8cb301a679ee9d5f36b6259801ff41cdadafc`. La descompilación de `entry`, evidencia `ev_3e3c93fd71a63dae9cbd9cf33d4b57b7fb4ee76e03cc6b197952c68d5a898c21`, muestra despacho de bookmarks, nueva pestaña, configuración, visor, comparación de carpetas, XDMCP y protocolo URI, junto con `FindWindowA("TMobaXtermForm", ...)`.

Las ramas de entrada recuperadas confirman que el programa actúa como coordinador y que contempla comunicación con una instancia existente. No se recorrieron exhaustivamente las 18.276 funciones. El pseudocódigo y sus límites se conservan en `rea-evidence-principal.json`; ambas sesiones nativas se cerraron después de exportar sus evidencias.

## Qué significa replicar para Linux

Se proponen dos objetivos diferentes que conviene decidir antes del desarrollo:

1. **Producto escrito en Rust con integración Linux:** interfaz y lógica propias, reutilizando PTY, OpenSSH, utilidades del sistema, Xwayland y backends de escritorio remoto. Reduce el trabajo duplicado y facilita compatibilidad inicial.
2. **Implementación íntegramente Rust:** implica además motores de terminal, protocolos, formatos, escritorio remoto y utilidades. Es un proyecto mucho mayor; no se obtiene por traducir los dos EXE principales.

No es necesario recrear Cygwin en Linux. Tampoco trasladar controles ActiveX, registro, instalador Windows o mecanismos HWND. Sí hay que preservar las capacidades que usa el usuario: sesiones, autenticación, transferencias, terminal, edición y escritorios remotos. El detalle de alternativas y fases se encuentra en `PROPUESTA_RUST_LINUX.md`.

## Criterios mínimos de aceptación funcional

| Componente | Prueba de aceptación |
|---|---|
| Terminal | Shell interactiva con resize, Unicode, teclas, señales y salida grande sin crecimiento ilimitado de RAM |
| SSH | Clave y agente, rechazo de host key cambiada, desconexión, reconexión y cierre sin procesos huérfanos |
| SFTP | Ficheros pequeños/grandes, nombres Unicode, permisos, cancelación y desconexión a mitad de transferencia |
| Edición remota | Detectar modificación externa, evitar sobrescritura silenciosa y limpiar temporales tras cierre/error |
| Túneles | Local/remoto/dinámico según alcance, puerto ocupado, cliente lento, salto y liberación de listeners |
| RDP/VNC | Login, resize, teclado, portapapeles y reconexión; cada redirección adicional se acepta por separado |
| Configuración | Escritura atómica, esquema versionado, recuperación de fichero corrupto y secretos fuera del archivo general |
| Macros | Orden de acciones, cancelación, destino visible y manejo de desconexión sin enviarlas a otra sesión |
| Procesos | Argumentos separados, ciclo de vida supervisado, salida acotada y cancelación del árbol propio |
| Portabilidad | Validación separada en X11 y Wayland; no asumir soporte de ventanas externas incrustadas |

## Lagunas que impiden afirmar equivalencia completa

- No se observó ejecución de los programas originales, ventanas, tráfico, servidores ni equipos remotos.
- No se han reconstruido todas las funciones nativas ni todos los caminos de despacho.
- Las cadenas y clases pueden incluir código de biblioteca no utilizado o funciones de otra edición.
- Las opciones internas pueden cambiar y no constituyen una especificación pública estable.
- No se ha demostrado la gramática completa de perfiles, macros ni IPC entre ejecutables.
- No se investigaron contraseñas guardadas, recuperación de secretos, activación ni mecanismos de licencia.
- La identidad de contenido derivado requiere conservar tanto el hash del original como el de su transformación.
- La propuesta Rust no es código implementado ni una prueba de que los backends cubran todas las prestaciones.

## Fuentes externas

- [Documentación oficial de MobaXterm](https://mobaxterm.mobatek.net/documentation.html): contratos públicos y categorías funcionales; no sustituye la evidencia de esta instalación.
- [Características oficiales](https://mobaxterm.mobatek.net/features.html): contraste de capacidades anunciadas.
- [REA](https://github.com/morluto/rea): herramientas de análisis y limitaciones de evidencia.

Las conclusiones de bytes locales se sustentan en los archivos de evidencia del expediente. Las propuestas de arquitectura son decisiones del análisis, no afirmaciones del fabricante.

## Verificación del expediente

- Se recalcularon los SHA-256 de los seis originales: coinciden con el inventario inicial; la instalación no se modificó.
- Se comprobó que el catálogo contiene las 485 rutas EXE registradas en el inventario de candidatos.
- Se validó la sintaxis JSON de los archivos de evidencia e inventarios y la codificación UTF-8 de los informes.
- Los tres documentos Markdown no contienen espacios al final de línea.
- No se modificó el repositorio HMI ni se hicieron commits. `git diff --check` no informó problemas.
- Los scripts auxiliares son una reproducción parcial del inventario y catálogo. El inventariador presupone los ZIP MOBASE/XW previamente derivados en `/tmp/mobaxterm-derived`; no es un extractor autónomo de toda la instalación. Los binarios derivados permanecen temporales y no son parte del informe distribuible.
- No se ejecutaron pruebas funcionales de la aplicación original ni de una réplica Rust, que todavía no está implementada.
