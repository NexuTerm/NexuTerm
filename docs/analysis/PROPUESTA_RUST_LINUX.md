# Propuesta de réplica funcional de MobaXterm en Rust para Linux

Fecha de consulta: 21 de octubre de 2026.
Estado: propuesta técnica; no se ha implementado ni probado esta aplicación.

## Objetivo y límites

Construir una aplicación Linux con los flujos de trabajo necesarios del producto analizado.
El catálogo estático de ejecutables sirve para decidir qué funcionalidades estudiar.
La presencia de un archivo, símbolo o cadena no demuestra que una función esté activa.
Esta propuesta no afirma equivalencia con la instalación de Windows.
Cada función necesita una prueba de aceptación observable y un conjunto de entradas.
Los componentes elegidos aquí son candidatos sujetos a una prueba de integración.

Hay dos objetivos técnicos distintos:

| Modalidad | Código propio | Dependencias externas | Consecuencia |
|---|---|---|---|
| Aplicación escrita en Rust | UI, sesiones, tareas y persistencia | GTK/VTE, OpenSSH, FreeRDP y programas Linux | Ruta inicial propuesta; integración práctica |
| Reimplementación integral en Rust | Además terminal, protocolos, codecs y utilidades | Sistema operativo y dependencias por concretar | Proyecto mucho mayor; requiere validar cada subsistema |

GTK/VTE mediante bindings Rust no constituye una implementación íntegra en Rust.
Invocar `ssh` o `rsync` tampoco reimplementa esos programas en Rust.
Una biblioteca Rust puede depender a su vez de código nativo: revisar features y árbol real.
No convertir el requisito de lenguaje de la aplicación en una promesa sobre toda la pila.

## Funciones que orientan el producto

Mobatek presenta sesiones, terminal, transferencia y escritorio remoto como funciones del producto.
También documenta túneles, edición remota, macros y herramientas auxiliares.
Estas descripciones identifican flujos que deben contrastarse con la instalación concreta.
Fuente: [funciones oficiales de MobaXterm](https://mobaxterm.mobatek.net/features.html).

## Matriz de implementación y aceptación

Todas las decisiones de esta tabla son propuestas del analista.
La columna de aceptación define trabajo pendiente, no pruebas ejecutadas.

| Capacidad | Implementación inicial para Linux | Alternativa con más Rust | Aceptación mínima |
|---|---|---|---|
| Terminal local por pestañas | GTK4 + widget VTE; shell del usuario en PTY | Motor y renderizador Rust propios | Bash y programa de pantalla completa; resize, UTF-8 y salida extensa |
| Sesiones guardadas | Modelo Rust y archivo de configuración versionado | Igual | Crear, editar, duplicar, reabrir y eliminar sin perder datos |
| SSH interactivo | OpenSSH como hijo de la terminal | `russh` y terminal integrada | Contraseña, clave, agente, rechazo de host cambiado y cierre |
| Jump host SSH | Configurar OpenSSH con destino y salto explícitos | Canales SSH encadenados | Host final accesible solo mediante bastión de laboratorio |
| Túneles local/remoto/dinámico | OpenSSH supervisado | Canales de forwarding con `russh` | Error de bind, reconexión deliberada y liberación del puerto |
| SFTP gráfico | Subsistema SFTP con API estructurada | `russh-sftp` | Listar, subir, bajar, renombrar y cancelar un archivo grande |
| SCP y sincronización | `scp` y `rsync` como procesos | Protocolo específico si realmente se necesita | Copias con espacios, Unicode, permisos y error parcial |
| X11 remoto | OpenSSH X11 forwarding y servidor gráfico Linux existente | Mantener la integración del sistema | Aplicación remota visible; cookies y cierre correctos |
| RDP | Cliente FreeRDP externo al principio | Integrar IronRDP | Autenticación, resize, teclado y canal solicitado por separado |
| VNC | Visor TigerVNC externo al principio | Cliente RFB propio o biblioteca evaluada | Imagen, teclado, portapapeles y política de autenticación |
| Puerto serie | Acceso Linux a dispositivo configurado | Gestión Rust del descriptor/termios | Baudrate, paridad, desconexión USB y reapertura |
| Telnet | Cliente del sistema supervisado | Negociación Telnet y terminal Rust | IAC, ecos, mensajes fragmentados y cierre |
| Rlogin/Rsh/XDMCP | Integraciones de compatibilidad opcionales | Reimplementar solo con una necesidad concreta | Matriz de equipos heredados especificada |
| Mosh | Lanzar cliente Linux existente | Reimplementación solo como proyecto separado | Migración de red y recuperación en laboratorio |
| Editor de texto | Editor Linux o widget especializado | Modelo de documento Rust | Codificación, fin de línea, búsqueda y aviso al cerrar sin guardar |
| Edición remota | Descargar, editar temporal y publicar por SFTP | Mismo flujo con backend Rust | Detectar modificación remota y no sobrescribir silenciosamente |
| Diff | Herramienta local o motor diff seleccionado | Algoritmo Rust detrás de una vista | Archivos grandes, binarios, CRLF/LF y diferencias Unicode |
| Macros | Secuencia explícita de acciones por sesión | Igual | Cancelación y exclusión de campos de credenciales |
| Envío a múltiples sesiones | Fan-out de entrada con selección visible | Igual | Destinos inequívocos, contrapresión y cancelación |
| Resaltado de terminal | Función del widget o capa de presentación | Parser y renderer propios | No alterar bytes enviados al proceso ni romper selección |
| Contraseñas | Referencias al almacén de secretos del escritorio | Integración D-Bus desde Rust | Bloqueo, desbloqueo y ausencia de secretos en config/log |
| Agente SSH | Agente existente del usuario | Implementar protocolo de agente si se justifica | Selección de clave y uso sin exportar material privado |
| Servidores auxiliares | Procesos Linux concretos bajo control del usuario | Servidores Rust por protocolo necesario | Bind explícito, parada y ausencia de procesos huérfanos |
| Utilidades Unix incluidas | Ejecutables de la distribución | Reescribir únicamente utilidades propias necesarias | Misma entrada/salida/código de retorno del contrato elegido |
| Navegador web auxiliar | Abrir navegador predeterminado | Widget web solo si hay requisito de integración | Abrir URL explícita y gestionar error de lanzamiento |
| Distribución de complementos | Paquetes Linux/dependencias declaradas | Formato propio solo si es necesario | Instalación reproducible y lista de versiones y licencias |

## Selección inicial de componentes

### Interfaz y terminal

La UI propuesta usa GTK4 desde Rust mediante gtk-rs.
El proyecto gtk-rs ofrece bindings para GTK4, GLib, Cairo y otras bibliotecas GNOME.
Esto permite una aplicación Rust con dependencias nativas explícitas.
Fuente: [gtk-rs](https://gtk-rs.org/).

GNOME VTE proporciona un widget de terminal y una abstracción de PTY.
La variante GTK4 declara el módulo pkg-config `vte-2.91-gtk4`.
Es una biblioteca LGPL, aspecto que se debe conservar en el inventario de distribución.
La documentación declara versión 0.78.2 en la página consultada; no fijar toda la pila sin un prototipo.
Fuente: [API oficial de VTE GTK4](https://gnome.pages.gitlab.gnome.org/vte/gtk4/).

No confundir GNOME VTE con el crate `vte` de Alacritty.
El crate de Alacritty es un parser de secuencias de terminal y no forma por sí solo un emulador.
Elegirlo exige añadir estado de pantalla, scrollback, selección, fuentes, IME y renderizado.
Fuente: [README de alacritty/vte](https://github.com/alacritty/vte).

### SSH y SFTP

OpenSSH ofrece cliente, agente, herramientas de claves, SFTP y SCP.
Su capa de conexión multiplexa canales y admite forwarding.
La primera implementación puede reutilizar la configuración del usuario mediante una opción explícita.
Debe conservar verificación de host y las peticiones de autenticación del cliente.
Fuente: [manuales oficiales de OpenSSH](https://www.openssh.org/manual.html).

`russh` es una biblioteca SSH asíncrona para Rust/Tokio.
Expone autenticación, canales y forwarding; su ecosistema incluye `russh-sftp`.
Es una alternativa si se necesita una API de sesión integrada y control de eventos.
El backend criptográfico se elige por features; revisar sus dependencias nativas antes de prometer Rust integral.
No cambiar a este backend hasta comparar la matriz de servidores, saltos y métodos de autenticación.
Fuente: [README oficial de russh](https://github.com/Eugeny/russh).

### Escritorio remoto

FreeRDP es una implementación de RDP bajo licencia Apache.
Usarlo como programa separado reduce el alcance inicial de la aplicación propia.
Integrar su biblioteca y mostrar el escritorio dentro de una pestaña necesita un prototipo distinto.
Fuente: [FreeRDP](https://www.freerdp.com/).

IronRDP implementa RDP mediante crates Rust para codecs, conexión, sesión y canales.
Su núcleo Sans-I/O necesita que la aplicación suministre transporte y runtime.
Su catálogo de protocolos no prueba que todos los canales funcionen en una UI concreta.
Evaluar teclado, gráficos, NLA, certificados y redirecciones de forma independiente.
Fuente: [README oficial de IronRDP](https://github.com/Devolutions/IronRDP).

TigerVNC ofrece cliente y servidor VNC multiplataforma y extensiones de autenticación/TLS.
Puede cubrir la apertura de sesiones VNC mediante un proceso externo.
Una ventana externa no equivale a integrar el framebuffer y eventos dentro de una pestaña.
Fuente: [TigerVNC](https://tigervnc.org/).

### Secretos

Guardar identificadores de secretos separados de la configuración ordinaria.
Secret Service define colecciones, elementos, bloqueo y solicitudes mediante D-Bus.
La especificación consultada se publica como borrador; comprobar el comportamiento del servicio instalado.
Si el almacén no está disponible, solicitar credencial por sesión en lugar de persistir texto plano.
Fuente: [Secret Service API](https://specifications.freedesktop.org/secret-service/latest/).

## Arquitectura mínima propuesta

Empezar con un ejecutable y módulos internos; no crear microservicios ni sistema de plugins.

```text
UI GTK4 escrita en Rust
  ├── sesiones y configuración versionada
  ├── terminal/PTY y procesos supervisados
  ├── transferencias con estado y cancelación
  ├── referencias al almacén de secretos
  └── tareas: SSH, túneles, editor y visores externos
```

Cada sesión tiene identificador propio, estado, procesos/tareas y recursos que debe cerrar.
La UI recibe eventos acotados; no espera operaciones lentas en el hilo gráfico.
Los procesos reciben argumentos como lista y no una cadena concatenada para `/bin/sh -c`.
Una orden que el usuario quiera ejecutar en su shell es un caso explícito diferente.
Aplicar límites a scrollback, transferencias simultáneas, logs y colas de eventos.
Copiar archivos por bloques, con progreso, cancelación y limpieza de temporales.
Cerrar una pestaña debe cancelar tareas, drenar eventos y recoger sus procesos hijos.
No tomar la reutilización de procesos externos como prueba de integración completa.
La elección de un runtime asíncrono global debe esperar al prototipo UI/transferencias.

## Persistencia y compatibilidad

Guardar sesiones sin contraseñas en el directorio de configuración de la aplicación.
Versionar el esquema desde la primera entrega y escribir mediante reemplazo atómico.
Usar un modelo propio legible con un número reducido de campos y defaults documentados.
La importación de MobaXterm debe empezar por perfiles de laboratorio con valores ficticios.
No asumir que las cadenas encontradas describen completamente su formato INI.
Mantener sin interpretar los campos no soportados al importar, o informar de su omisión.
No declarar importación de credenciales hasta tener especificación y pruebas independientes.
La migración inicial puede importar host, puerto, usuario, tipo y nombre sin copiar secretos.

## Entrega por fases

| Fase | Entregable | Criterio de salida |
|---|---|---|
| 0 | Inventario y casos de uso priorizados | Cada función requerida tiene entrada, resultado y prueba definida |
| 1 | UI, sesiones y terminal local | Shell, aplicación TUI, resize y cierre sin procesos huérfanos |
| 2 | SSH y túneles | Host keys, claves/agente, salto, forwarding y errores cubiertos |
| 3 | SFTP y edición remota | Transferencia cancelable y conflicto de edición resuelto explícitamente |
| 4 | RDP y VNC externos | Apertura y cierre supervisados; límites de integración visibles |
| 5 | Integración gráfica y herramientas necesarias | Canales y acciones aprobados por caso de uso |
| 6 | Empaquetado y estabilidad | Instalación limpia, dependencias declaradas y pruebas prolongadas |

No asignar fechas de entrega a partir del número de EXEs.
Muchos ejecutables son alias, herramientas generales o compatibilidad específica de Windows.
La estimación exige cerrar primero el catálogo funcional y los protocolos prioritarios.

## Pruebas que faltan antes de afirmar equivalencia

- Terminal: colores, Unicode ancho/combinado, IME, bracketed paste y aplicación de pantalla completa.
- Sesiones: perfiles simultáneos, error parcial al cargar y recuperación de escritura interrumpida.
- SSH: host desconocido, clave cambiada, clave cifrada, agente y keyboard-interactive.
- Red: conexión lenta, timeout, desconexión, salida masiva y cancelación durante autenticación.
- Túneles: puertos ocupados, clientes lentos y cierre de la pestaña mientras hay conexiones activas.
- SFTP: directorios extensos, archivos grandes, enlaces, permisos y servidor que cambia el archivo.
- RDP/VNC: layouts de teclado, escalado, resize y cada redirección que se pretenda ofrecer.
- Serie: dispositivo ausente, desconexión física y permisos denegados.
- Recursos: CPU, RSS/PSS, descriptores y procesos durante carga, reposo y cierre repetido.
- Distribución: instalación y desinstalación en una VM Linux limpia con versiones documentadas.

Estas son pruebas planificadas. No se ejecutaron como parte del inventario estático.
El primer producto útil puede cubrir terminal, SSH, SFTP y túneles sin reproducir todas las utilidades.
Las fases posteriores deben responder a necesidades verificadas y no al deseo de traducir cada EXE a Rust.
