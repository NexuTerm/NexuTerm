# Comprobación de conexiones de IA

Los agentes se configuran en NexuTerm con su ejecutable o endpoint, modelo y
credenciales correspondientes. La disponibilidad depende del proveedor, de su
catálogo y de la autenticación de cada perfil. No se distribuyen conexiones
personales, sesiones autenticadas ni modelos instalados como parte de la aplicación.

## Prueba optativa con proveedores

El ejemplo `examples/check_agent_connections.rs` utiliza el cliente del chat:

```bash
cargo run --example check_agent_connections --locked --offline --jobs 2
```

Su alcance está fijado en el ejemplo:

- Consulta los perfiles Codex de la configuración local de NexuTerm.
- Prueba OpenCode con un perfil temporal y hasta dos modelos que el catálogo
  identifique como gratuitos según los filtros del ejemplo.
- Prueba el endpoint Ollama local `http://127.0.0.1:11434/v1` con
  `qwen2.5-coder:1.5b-base`. Ese modelo y el servidor deben existir previamente.

El comando puede enviar consultas reales y consumir cuota del proveedor. No
instala modelos, no inicia sesión automáticamente ni modifica los perfiles.
Utiliza carpetas de trabajo temporales, desactiva el acceso a terminales y
rechaza solicitudes de herramientas. `--offline` afecta a Cargo, no impide
las conexiones de red del programa.

El resultado distingue conexión, catálogo, texto recibido y errores. Consultar
un catálogo no demuestra que todos sus modelos generen respuestas; recibir texto
no mide su calidad. El ejemplo no cubre todos los proveedores configurables ni
sustituye las pruebas de permisos, cancelación e historial del chat.

## Registro de una comprobación

Registra fecha, versión de NexuTerm, proveedor, modelo, resultado y alcance sin
incluir credenciales, tokens ni historiales privados. No extrapoles resultados
de una cuenta o fecha a todas las instalaciones. Las comprobaciones automatizadas
y sus límites se describen en [VALIDACION.md](VALIDACION.md); la aceptación
operativa, en [ACEPTACION.md](ACEPTACION.md).
