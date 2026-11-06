# Herramientas y dependencias de NexuTerm

Revisión documental: 26 de octubre de 2026. Este catálogo describe dependencias
del proyecto; no certifica su instalación en el equipo actual.

| Componente | Uso | Necesidad |
| --- | --- | --- |
| Rust/Cargo >= 1.92 | Compilar y ejecutar pruebas Rust | Desarrollo |
| `pkg-config`, compilador C | Detectar bibliotecas y compilar el puente VNC | Compilación |
| `glib-compile-resources` | Integrar iconos y recursos GTK | Compilación con interfaz |
| GTK >= 4.10, libadwaita >= 1.4, VTE GTK4 >= 0.70 | Interfaz y terminal | Aplicación gráfica |
| LibVNCClient | Backend VNC mediante enlace nativo | Compilación y ejecución |
| OpenSSL/bibliotecas TLS del sistema | Transporte TLS | Compilación y ejecución |
| OpenSSH: `ssh` | Terminal, SFTP y edición remota | Conexiones SSH/SFTP |
| `telnet` | Terminal Telnet | Conexiones Telnet |
| GVfs con backend FTP | Navegación y transferencias FTP | Conexiones FTP |
| `curl` | Subidas FTP, compatibilidad Windows CE, API e imágenes HTTP | Funciones correspondientes |
| Secret Service y proveedor de llavero | Persistencia opcional de credenciales | Guardado de secretos |
| CLI o adaptador del agente configurado | Transporte nativo de chat | Opcional por perfil |
| `pw-record`, `whisper-cli` y modelo compatible | Dictado local | Opcional |
| Python >= 3.11, `dpkg-dev`, `binutils` | Generación del paquete Debian | Empaquetado |
| Python, Xvfb y D-Bus | Comprobaciones gráficas aisladas | Pruebas de laboratorio |
| `sshd`, `ssh-keygen`, `ssh` | Servidor y claves temporales | Pruebas de edición remota |

RDP utiliza IronRDP compilado en el programa. VNC utiliza LibVNCClient.
No requieren visores de escritorio remoto externos.
Las bibliotecas Rust y sus versiones resueltas se consultan en `Cargo.toml`
y `Cargo.lock`; no se duplican aquí.

Para revisar dependencias del binario de release:

```bash
make check-runtime
```

Consulta el [README](../../README.md) para instalar, compilar y empaquetar.
Las dependencias de sistema exactas del paquete se calculan con
`dpkg-shlibdeps` en `packaging/release.py`.
