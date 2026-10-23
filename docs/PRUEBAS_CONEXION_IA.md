# Pruebas reales de conexión de IA — 23 de octubre de 2026

Se enviaron mensajes mínimos sin herramientas ni acceso al proyecto. No se
modificaron los perfiles del usuario ni se instalaron o actualizaron proveedores.

| Proveedor/modelo | Resultado observado |
| --- | --- |
| Codex, perfil configurado de NexuTerm | El servidor respondió; cuenta sin sesión autenticada. No se envió inferencia. |
| OpenCode / opencode/beam-free | Inicialización ACP y catálogo correctos. La inferencia falló: el servicio exige OpenCode >= 1.18.0; versión instalada 1.17.15. |
| OpenCode / opencode/big-pickle | Mismo rechazo de versión al enviar el mensaje mediante ACP. |
| Ollama / qwen2.5-coder:1.5b-base | Petición Chat Completions completada con texto no vacío mediante el cliente de NexuTerm. |
| GitHub Copilot | Perfil configurado, pero ejecutable `copilot` ausente. No se pudo conectar. |
| Claude Code y Gemini CLI | Ejecutables/adaptadores ausentes. No se pudo conectar. |
| DeepSeek y otras API externas | No hay perfiles configurados para probarlas. |

Ollama publicó nueve modelos mediante `/v1/models`; solo se ejecutó el modelo
pequeño indicado. El acceso al catálogo no demuestra inferencia en los demás.
La respuesta de Ollama verifica transporte y generación, no calidad del modelo.

La prueba optativa reproducible utiliza el mismo `codex::Client` del chat:

```bash
cargo run --example check_agent_connections --locked --offline --jobs 2
```

Este comando consume una consulta mínima cuando encuentra una conexión operativa.
No inicia sesión automáticamente. Usa un directorio de trabajo temporal y rechaza
las solicitudes de herramientas. OpenCode utiliza un perfil temporal sin claves.
El diagnóstico adicional ACP directo confirmó el error de versión en los dos
modelos gratuitos anteriores, sin imprimir credenciales.

## Ampliación de conexiones y acceso web

Se instalaron y comprobaron cinco modelos locales adicionales con respuestas de
texto no vacías por Chat Completions: `qwen3:0.6b`, `qwen3:1.7b`,
`llama3.2:1b`, `qwen2.5-coder:1.5b` y `tinyllama:1.1b`. Sus conexiones están
guardadas con esos nombres exactos. Gemma 3 1B no se añadió: el registro exigió
una versión de Ollama posterior a la instalada (0.17.4).

Se instalaron Gemini CLI 0.65.0 y Copilot CLI 1.0.103 en
`.local/ai-providers/`, excluido de Git; no se instalaron globalmente.

- Copilot: autorización web confirmada, catálogo ACP consultado y respuesta real
  de texto completada (`end_turn`). La cuenta anunció únicamente `auto` / `Auto`.
  Se guardó `Auto · GitHub Copilot` con su directorio privado ya autenticado.
  La disponibilidad gratuita depende del plan y de su cuota; no se contrató nada.
- Google: caché OAuth recibida en el perfil privado, sin leer ni imprimir sus
  credenciales. El proveedor rechazó después el cliente para cuentas individuales
  con un mensaje de migración a Antigravity. No se guardó conexión Gemini bloqueada.
- Codex se conservó intacto. No se inició ni modificó su autenticación durante
  esta ampliación.

Se guardaron en total trece perfiles: Codex, once modelos locales comprobados y
Copilot Auto. No se añadieron conexiones que fallaron ni alias locales duplicados.
