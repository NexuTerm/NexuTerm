# Alcance y trabajo pendiente de NexuTerm

Revisión documental: 4 de noviembre de 2026 — versión 1.0.0. Esta lista distingue funciones
documentadas de ampliaciones pendientes; no implica fechas ni aprobación para
implementar nuevas funciones.

## Alcance actual

El [README](../../README.md) describe terminales locales, sesiones SSH/SFTP,
Telnet, FTP, RDP y VNC, exploradores de archivos, editor y chat de agentes.
La [arquitectura](ARQUITECTURA.md) identifica los módulos que las implementan.
El [historial de validación](../VALIDACION.md) recoge las comprobaciones
realizadas, sus entornos y sus limitaciones.

## Límites y ampliaciones pendientes

- Ampliar la ejecución simultánea general y las métricas continuas. Los túneles
  SSH y la ejecución supervisada ya tienen interfaz.
- Macros, Mosh, conexiones serie y servidores auxiliares no están implementados.
  La comparación de versiones y la importación estática de OpenSSH sí están disponibles.
- La cola de transferencias admite árboles de carpetas, pausa, cancelación y
  reintento desde el inicio de cada archivo, sin sobrescribir destinos existentes.
  No es sincronización bidireccional ni reanudación por offset de un archivo parcial.
- El editor acepta texto UTF-8 hasta 8 MiB. La edición remota detecta conflictos,
  pero no bloquea escrituras de otros clientes durante la subida.
- RDP/VNC todavía requieren ampliar la validación de servidores, teclado y
  desconexión. No se ofrece sincronización de portapapeles ni audio.
- Los adaptadores de agentes no ofrecen necesariamente las mismas funciones.
  Los catálogos y el acceso real dependen del proveedor y su autenticación.
- La distribución Flatpak/AppImage y los runtimes para sistemas antiguos siguen
  pendientes. La compatibilidad debe probarse en cada distribución objetivo.

## Criterios para aceptar cambios

Cada ampliación debe tener un caso de uso concreto, cambios acotados y pruebas
reproducibles. Los protocolos requieren comprobar autenticación, desconexión,
entradas inválidas y cierre de recursos con servidores de laboratorio.
Las pruebas de estabilidad deben registrar carga, duración, errores, memoria,
descriptores y procesos tras periodos de reposo.

No se deben interpretar los resultados históricos como validación del código
actual ni usar servicios de producción para pruebas destructivas.
