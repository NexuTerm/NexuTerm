# Aceptación operativa

La hoja de ruta tiene implementación y pruebas automatizadas. La aceptación por
personas de soporte/administración es una comprobación distinta y no se sustituye
por compilaciones, simuladores de proveedores o capturas automatizadas. Los
resultados fechados se conservan en [VALIDACION.md](VALIDACION.md).

## Matriz nativa reproducible

`lab/check_distributions.py` crea contenedores Podman propios de Ubuntu 24.04 y
Debian 13, copia una instantánea de fuentes, registra su SHA-256 y las versiones
nativas y ejecuta pruebas, GTK/Xvfb, SSH real en loopback, compilación optimizada,
comprobación de dependencias y creación del paquete Debian. No monta credenciales,
configuración personal ni publica puertos. Necesita Podman, acceso a APT y una
instalación de Rust compatible con su registro Cargo ya descargado:

```bash
python3 lab/check_distributions.py \
  --toolchain /ruta/toolchain-rust \
  --registry /ruta/cargo/registry \
  --output-dir /ruta/resultados \
  --release
```

`--image ubuntu:24.04` o `--image debian:13` limita la distribución;
`--resume` reutiliza únicamente las cachés de una matriz anterior, creando otra
instantánea. `--keep-failed` conserva el contenedor propio fallido para diagnóstico;
sin esa opción se elimina al terminar. Los logs, `result.json`, `matrix.json`,
capturas y paquetes permanecen en el directorio de resultados. No se deben borrar
contenedores ajenos ni reutilizar cuentas de producción para ejecutar el laboratorio.

Las pruebas gráficas usan X11 virtual y herramientas locales aisladas. No certifican
Wayland, hardware concreto, toda política SSH de terceros ni instalación/actualización
del paquete sobre un escritorio personal. El selector nativo de exportación necesita
además comprobación manual; el editor de informe y la escritura privada sin
sobrescritura sí se ejercitan automáticamente.

## Medición del flujo de trabajo

`python3 lab/check_remote_editor.py` imprime `NEXUTERM_BENCHMARK` mediante la prueba
Rust de SSH. Mide preparación del ControlMaster, planificación y copia SFTP de una
carpeta de 32 archivos (262144 bytes, cuatro subcarpetas), recuperación de su cola
persistida y ejecución de la receta de disco con GNU df real. Se comprueban los
bytes de cada archivo y que recuperar la cola completada no repite trabajos.

Una ejecución local el 2 de noviembre de 2026 dio: preparación SSH 1015,897 ms,
planificación 9,982 ms, copia 471,970 ms, recuperación de cola 7,778 ms y diagnóstico
16 ms con retorno 0. Son observaciones de una ejecución en loopback, no un benchmark
estadístico ni una promesa de rendimiento en redes o discos distintos. Los logs de
cada distribución permiten contrastar sus propias medidas.

Las mismas comprobaciones en contenedores nativos dieron estas medidas (ms):

| Entorno | Preparación SSH | Planificación | Copia SFTP | Recuperación de cola | Diagnóstico |
| --- | ---: | ---: | ---: | ---: | ---: |
| Ubuntu 24.04 | 1010,441 | 9,032 | 492,227 | 10,583 | 10 |
| Debian 13 | 1015,145 | 10,682 | 567,080 | 8,727 | 13 |

En ambos casos se verificaron los 32 archivos y el diagnóstico terminó con retorno
0. La carga y el alcance son los mismos descritos arriba; no se interpreta la
diferencia entre ejecuciones como una comparación estadística de distribuciones.

## Comprobación pendiente con operadores y cuentas de prueba

En un equipo de prueba, con datos ficticios y destinos desechables:

1. Un operador de soporte y un administrador deben crear una intervención con
   objetivo y destinos claros, registrar el diagnóstico y revisar una propuesta.
2. Rechazar una ejecución y comprobar que no se realiza. Aprobar después una
   escritura únicamente en una carpeta temporal propia, y verificar su resultado
   mediante otra ejecución vinculada. Confirmar que las notas de aprobación no
   conceden permisos y que el destino y comando revisados son los ejecutados.
3. Recuperar un borrador tras cerrar y tras una interrupción; comprobar que los
   resultados no confirmados se reconocen y que no se repiten acciones.
4. Revisar el informe sin salidas/comandos, excluir una entrada, redactar texto
   ficticio sensible y exportar Markdown/JSON. Comprobarlo antes de compartirlo.
   Borrar el contexto y confirmar que no desaparece su historial independiente.
5. Con cuentas de prueba autorizadas de cada proveedor de IA configurado, abrir
   chats recientes del agente, iniciar/reanudar y cancelar una conversación, y
   comprobar permisos supervisados/acceso completo con una acción temporal inocua.
   Las pruebas sintéticas del protocolo no acreditan disponibilidad, autenticación
   ni comportamiento de los servicios reales.
6. Registrar distribución, backend gráfico, versión de NexuTerm, participantes,
   casos, resultados y defectos. La conformidad de uso debe darla cada participante;
   no se presupone aprobación por ausencia de incidencias automatizadas.

No se han ejecutado estas comprobaciones humanas ni se han usado cuentas reales
para cerrar esta entrega. No hay autorización para probar contra equipos
industriales o servicios de producción.
