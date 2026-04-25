---
continuity_session: CONT-2026-04-25-1605-implement-rtasks-v1
created_at: 2026-04-25 16:05
updated_at: 2026-04-25 16:05
status: active
goal: Implementar RTasks V1 completo según rtasks.md
---

# AUTONOMOUS_EXECUTION.md

## Purpose

Definir el contrato operativo para implementar RTasks V1 de forma autónoma, auditable, pausable y reanudable.

RTasks V1 es una app desktop Windows mínima para:

1. Capturar tareas en menos de 2 segundos.
2. Mostrar tareas activas en un panel mínimo.

## Session metadata

- Session: `CONT-2026-04-25-1605-implement-rtasks-v1`
- Status: `active`
- Goal: implementar RTasks V1 completo según `rtasks.md`.
- Stack objetivo: Rust + `eframe/egui` + JSONL.
- Hotkeys definitivas:
  - Quick Add: `Alt+Spacebar`
  - Panel: `Ctrl+Spacebar`

## Source of truth

- `AUTONOMOUS_EXECUTION.md` = reglas del juego.
- `ACTIVE_QUEUE.md` = cola/tareas/dependencias.
- `STATE.md` = bitácora/checkpoints.
- `rtasks.md` = spec funcional y técnica del producto.

Si hay conflicto, prioridad:

1. Instrucciones explícitas del usuario.
2. `rtasks.md`.
3. `ACTIVE_QUEUE.md`.
4. `STATE.md`.
5. Este contrato.

## Command chain

```text
/init-cont  -> crea sesión + contrato + estado + cola base
/plan-cont  -> refina ACTIVE_QUEUE.md con plan completo
/start-cont -> ejecuta ACTIVE_QUEUE.md
/fin-cont   -> archiva sesión y genera commit message
```

## Triggers

Continuar esta sesión cuando el usuario diga:

- `/plan-cont`
- `/start-cont`
- `continúa`
- `seguí con RTasks`
- `implementa RTasks`
- `no pares hasta terminar`
- `finaliza continuidad`
- `/fin-cont`

## Scope and autonomy level

Autonomía permitida: implementar V1 local, chica y confiable.

Dentro de scope:

- Crear proyecto Rust.
- Implementar modelo de tareas.
- Implementar storage JSONL seguro.
- Implementar parser mínimo.
- Implementar UI Quick Add.
- Implementar UI Panel.
- Implementar hotkeys globales.
- Validar con comandos locales razonables.
- Actualizar `ACTIVE_QUEUE.md` y `STATE.md` después de cada tarea relevante.

Fuera de scope:

- Producto público.
- Installer.
- Autoupdate.
- Sync.
- Login.
- IA/LLM.
- Plugins.
- Tags custom.
- Calendario visual.
- Integraciones.

## Allowed actions

- Leer archivos relevantes del proyecto.
- Crear/modificar archivos necesarios para RTasks V1.
- Ejecutar comandos de inspección (`ls`, `rg`, `find`, `cargo check`, etc.).
- Crear estructura Rust si no existe.
- Ajustar dependencias compatibles si una versión del spec no compila.
- Documentar decisiones técnicas breves en `STATE.md`.

## Forbidden actions

- No implementar features prohibidas en `rtasks.md`.
- No usar Tauri, webview, Supabase, Anytype ni backend.
- No introducir SQLite en V1.
- No usar IA/LLM para parseo de fechas.
- No borrar progreso existente sin confirmación.
- No hacer commit sin pedido explícito del usuario.
- No usar `git add -A` ni `git add .`.
- No usar `git reset --hard`.
- No usar `git checkout .`.
- No usar `git clean -fd`.
- No usar `git stash` salvo confirmación explícita.
- No tocar cambios de otros agentes.

## Validation commands

Como este directorio todavía no tiene proyecto Rust, los comandos se habilitan por fase:

- Después de crear proyecto: `cargo check` desde la raíz del crate.
- Si se agregan tests: ejecutar tests específicos creados/modificados.
- Validación manual esperada:
  - `Alt+Spacebar` abre Quick Add desde otra app.
  - `Ctrl+Spacebar` abre/cierra Panel desde otra app.
  - `Enter` guarda tarea.
  - JSONL persiste entre ejecuciones.
  - Panel muestra `TODO` y `DOING`, no `DONE`.

No correr comandos destructivos. No correr builds largos salvo necesidad explícita.

## Execution loop

Para `/start-cont`:

1. Leer `AUTONOMOUS_EXECUTION.md`, `ACTIVE_QUEUE.md`, `STATE.md` y `rtasks.md`.
2. Verificar que los tres archivos de continuidad compartan `continuity_session`.
3. Tomar la primera tarea `pending` cuyas dependencias estén `done`.
4. Marcarla `in_progress` con timestamp.
5. Implementar el mínimo necesario para cumplir su DoD.
6. Validar.
7. Marcar `done`, `partial` o `blocked`.
8. Actualizar `STATE.md` con checkpoint.
9. Continuar con la siguiente tarea hasta completar la cola o encontrar bloqueo real.

## Stop conditions

Detener ejecución si:

- Hay conflicto de sesión entre archivos de continuidad.
- Falta información esencial no inferible desde `rtasks.md`.
- Una dependencia externa impide compilar o validar.
- Un comando requiere permiso destructivo.
- Hay riesgo de pisar cambios ajenos.
- La implementación requiere salir del scope V1.

## Checkpointing rules

Actualizar `STATE.md` cuando:

- Se crea o cambia estructura de proyecto.
- Se completa una tarea de `ACTIVE_QUEUE.md`.
- Se detecta bloqueo.
- Se cambia una decisión técnica.
- Se ejecuta una validación importante.

Cada checkpoint debe incluir:

- Timestamp.
- Tarea activa o recién completada.
- Archivos tocados.
- Resultado de validación.
- Próximo paso.

## Queue rules

- Valores de estado: `pending`, `in_progress`, `done`, `blocked`, `partial`, `cancelled`.
- Nunca renumerar IDs existentes.
- Elegir la primera tarea `pending` con dependencias `done`.
- Preservar claims salvo stale o override explícito.
- Si una tarea se vuelve demasiado grande, dividirla agregando IDs nuevos al final.
- No marcar `done` sin DoD cumplido y validación razonable.

## Claim rules

- `Claimed by` debe indicar `pi-agent` cuando una tarea esté en progreso.
- `Started` se completa al iniciar.
- `Last update` cambia al modificar estado.
- Si una tarea queda bloqueada, documentar bloqueo en `Notes` y `STATE.md`.

## Archive/finalization rules

Para `/fin-cont`:

- Marcar sesión como `archived` o `paused` según corresponda.
- Crear snapshot en carpeta de archivo si el skill lo pide.
- Dejar `STATE.md` con estado final.
- Generar commit message sugerido, sin commitear salvo pedido explícito.

## Reporting format

Responder breve:

- Tarea trabajada.
- Archivos cambiados.
- Validación ejecutada.
- Estado: `done`, `partial` o `blocked`.
- Próximo paso.

## Project-specific task map

Fases principales:

1. Skeleton Rust.
2. Modelo + storage JSONL.
3. Parser mínimo.
4. App state.
5. Quick Add UI.
6. Panel UI.
7. Hotkeys globales.
8. Pulido mínimo.
9. Validación final MVP.

## Re-entry instructions

Al retomar:

1. Leer `STATE.md`.
2. Leer front matter de `AUTONOMOUS_EXECUTION.md` y `ACTIVE_QUEUE.md`.
3. Confirmar misma `continuity_session`.
4. Leer `rtasks.md` si hace falta contexto funcional.
5. Continuar desde la primera tarea `pending` en `ACTIVE_QUEUE.md`.

## Commit policy

- No commitear sin pedido explícito.
- Si el usuario pide commit: stagear solo archivos tocados en esta sesión.
- Nunca usar `git add -A` ni `git add .`.
- Verificar estado antes de commit.
- No force push.
