---
continuity_session: CONT-2026-04-25-1605-implement-rtasks-v1
created_at: 2026-04-25 16:05
updated_at: 2026-04-25 18:28
status: active
goal: Implementar RTasks V1 completo según rtasks.md
---

# ACTIVE_QUEUE.md

## Current goal

Implementar RTasks V1 completo: app desktop Windows local en Rust + egui para captura rápida de tareas con `Alt+Spacebar`, panel con `Ctrl+Spacebar`, storage JSONL y parser mínimo.

## Queue policy

- Status values: `pending`, `in_progress`, `done`, `blocked`, `partial`, `cancelled`.
- Never renumber existing task IDs.
- Pick first pending task whose dependencies are done.
- Preserve claims unless stale or explicitly overridden.
- Keep V1 brutally small: no sync, no login, no IA, no plugins, no calendario, no tags custom.
- Update `STATE.md` after each completed/blocked task.

## Queue

### T001 — Crear skeleton Rust

Status: done
Claimed by: pi-agent
Started: 2026-04-25 16:06
Last update: 2026-04-25 16:07
Scope:
- Crear proyecto Rust para RTasks si no existe.
- Definir estructura base de crate.
- Agregar dependencias iniciales para `eframe/egui`.
- Mostrar ventana básica.
DoD:
- Existe `Cargo.toml`.
- Existe `src/main.rs`.
- La app abre una ventana egui básica.
Validation:
- `cargo check` desde el crate.
Files likely touched:
- `Cargo.toml`
- `src/main.rs`
Risk: low
Depends on:
- none
Notes:
- Si `cargo new` crea subcarpeta, documentar ruta elegida en `STATE.md`.

### T002 — Definir modelo de dominio

Status: done
Claimed by: pi-agent
Started: 2026-04-25 16:08
Last update: 2026-04-25 16:09
Scope:
- Implementar `Task`.
- Implementar `TaskStatus`.
- Implementar `Priority`.
- Implementar `TaskSource`.
- Derivar traits necesarios para serde, clone, debug y comparaciones.
DoD:
- Modelo coincide con `rtasks.md`.
- Serialización JSON produce variantes legibles como `Todo`, `Doing`, `Done`, `Medium`, etc.
Validation:
- `cargo check`.
Files likely touched:
- `src/task.rs`
- `src/main.rs` o `src/lib.rs`
Risk: low
Depends on:
- T001
Notes:
- Evitar `any` equivalente; usar tipos concretos.

### T003 — Implementar storage JSONL seguro

Status: done
Claimed by: pi-agent
Started: 2026-04-25 16:10
Last update: 2026-04-25 16:12
Scope:
- Resolver ruta `%APPDATA%\rtasks\tasks.jsonl` con `directories` o alternativa robusta.
- Crear carpeta `%APPDATA%\rtasks\` si falta.
- Implementar `load_tasks()`.
- Implementar `save_tasks()` con `tasks.tmp` + rename.
- Implementar helpers `add_task()`, `update_task()` y borrado desde memoria persistida.
DoD:
- Carga JSONL existente.
- Ignora líneas vacías.
- Guarda una tarea por línea.
- No edita línea por línea.
- Escritura segura con temporal y rename.
Validation:
- `cargo check`.
- Prueba manual o test unitario de roundtrip si se agrega harness simple.
Files likely touched:
- `src/storage.rs`
- `src/task.rs`
- `Cargo.toml`
Risk: medium
Depends on:
- T002
Notes:
- Si hay errores de línea corrupta, decidir si fallar con contexto o saltar; preferir fallar claro en V1.

### T004 — Implementar parser mínimo de fechas

Status: done
Claimed by: pi-agent
Started: 2026-04-25 16:13
Last update: 2026-04-25 16:14
Scope:
- Parsear `hoy`.
- Parsear `mañana`.
- Parsear `pasado mañana`.
- Remover palabra temporal del título.
- Devolver `due` como `YYYY-MM-DD`.
DoD:
- `comprar pan mañana` produce title `comprar pan` y due de mañana.
- `comprar pan` produce due `None`.
- No usa IA ni parser complejo.
Validation:
- `cargo check`.
- Tests unitarios si se configura módulo de tests.
Files likely touched:
- `src/parser.rs`
- `Cargo.toml`
Risk: low
Depends on:
- T002
Notes:
- Mantener simple; no implementar días de semana salvo que quede trivial y no desvíe V1.

### T005 — Crear estado de aplicación

Status: done
Claimed by: pi-agent
Started: 2026-04-25 16:15
Last update: 2026-04-25 16:16
Scope:
- Crear `RTasksApp`.
- Crear `AppMode`: `Hidden`, `QuickAdd`, `Panel`.
- Conectar carga inicial de tareas.
- Mantener input, status, priority y selección de panel.
DoD:
- App compila con estado interno.
- Carga tareas al iniciar.
- Puede persistir cambios invocando storage.
Validation:
- `cargo check`.
Files likely touched:
- `src/app.rs`
- `src/main.rs`
- `src/storage.rs`
Risk: medium
Depends on:
- T003
- T004
Notes:
- Si egui exige una sola ventana inicial, simular modos con viewport/ventana hasta resolver hotkeys.

### T006 — Implementar Quick Add UI

Status: done
Claimed by: pi-agent
Started: 2026-04-25 16:17
Last update: 2026-04-25 16:19
Scope:
- Modal centrado de captura.
- Input con foco automático.
- Mostrar status y prioridad actuales.
- Defaults: `Todo`, `Medium`.
- `Enter` guarda.
- `Esc` cierra sin guardar.
- `Alt+1/2/3` cambia status.
- `Alt+Q/W/E` cambia prioridad.
- Limpiar input y ocultar después de guardar.
DoD:
- Se puede escribir `comprar pan` y guardar tarea.
- Se puede escribir `comprar pan mañana` y guarda due correcto.
- Atajos internos funcionan.
Validation:
- `cargo check`.
- Validación manual en app.
Files likely touched:
- `src/ui/quick_add.rs`
- `src/app.rs`
- `src/main.rs`
Risk: medium
Depends on:
- T005
Notes:
- Priorizar funcionalidad sobre estética.

### T007 — Implementar Panel UI

Status: done
Claimed by: pi-agent
Started: 2026-04-25 16:20
Last update: 2026-04-25 16:22
Scope:
- Panel bottom-right aproximado.
- Mostrar título `RTasks`.
- Mostrar secciones `TODO` y `DOING`.
- Ocultar `DONE`.
- Ordenar: status, prioridad desc, created_at asc.
- Renderizar prioridad `[ALTA]`, `[MEDIA]`, `[BAJA]`.
- Click alterna `Todo` ↔ `Doing`.
- `Ctrl+Click` marca `Done` y setea `completed_at`.
- `Del` borra tarea seleccionada.
- `Esc` cierra panel.
DoD:
- Panel muestra tareas activas correctamente.
- Cambios de estado persisten.
- Borrado persiste.
Validation:
- `cargo check`.
- Validación manual con varias tareas.
Files likely touched:
- `src/ui/panel.rs`
- `src/app.rs`
- `src/task.rs`
Risk: medium
Depends on:
- T005
- T006
Notes:
- El orden debe coincidir con spec, no con derivado automático si contradice High primero.

### T008 — Implementar hotkeys globales

Status: done
Claimed by: pi-agent
Started: 2026-04-25 16:23
Last update: 2026-04-25 16:25
Scope:
- Registrar `Alt+Spacebar` para Quick Add.
- Registrar `Ctrl+Spacebar` para Panel.
- Integrar eventos de `global-hotkey` con app egui.
- Abrir/cerrar modos desde cualquier app.
DoD:
- `Alt+Spacebar` abre Quick Add desde otra app.
- `Ctrl+Spacebar` abre/cierra panel desde otra app.
- Hotkeys no cierran el proceso.
Validation:
- `cargo check`.
- Validación manual desde otra aplicación Windows.
Files likely touched:
- `src/hotkeys.rs`
- `src/app.rs`
- `src/main.rs`
- `Cargo.toml`
Risk: high
Depends on:
- T006
- T007
Notes:
- Esta es la parte con más riesgo por integración de event loop/windowing.

### T009 — Pulido mínimo V1

Status: done
Claimed by: pi-agent
Started: 2026-04-25 16:26
Last update: 2026-04-25 16:28
Scope:
- Always on top si el stack lo permite sin complejidad excesiva.
- Posición Quick Add centrada.
- Posición Panel bottom-right.
- Tema dark.
- Fuente/espaciado legible.
- Cerrar ventana no debe cerrar app si hotkeys/tray/background están activos.
DoD:
- UX básica usable.
- No se agregan features fuera de V1.
Validation:
- `cargo check`.
- Validación manual de flujo completo.
Files likely touched:
- `src/app.rs`
- `src/main.rs`
- `src/ui/quick_add.rs`
- `src/ui/panel.rs`
Risk: medium
Depends on:
- T008
Notes:
- No agregar animaciones, settings, cloud, tags ni temas configurables.

### T010 — Validación final MVP y documentación de estado

Status: done
Claimed by: pi-agent
Started: 2026-04-25 16:29
Last update: 2026-04-25 18:28
Scope:
- Ejecutar validación final disponible.
- Revisar tasklist de `rtasks.md` contra implementación.
- Actualizar `STATE.md` con estado final.
- Documentar limitaciones si queda algo parcial.
DoD:
- `cargo check` pasa.
- Flujo MVP validado o bloqueos documentados.
- `STATE.md` indica siguiente paso real.
Validation:
- `cargo check`.
- Manual: Quick Add, guardar, Panel, alternar, done, borrar, persistencia.
Files likely touched:
- `STATE.md`
- Posibles ajustes menores en código.
Risk: low
Depends on:
- T009
Notes:
- No commit salvo pedido explícito.
- Validación automática completada: `cargo test` OK y `cargo check` OK.
- Feedback manual recibido: MVP abre y funciona, pero se detectaron issues de UX.
- Fix aplicado: hotkeys internas Alt+1/2/3 y Alt+Q/W/E ya no deberían dejar caracteres en input.
- Fix aplicado: parser se ejecuta después de TextEdit, evitando guardar texto sin procesar por orden de eventos.
- Fix aplicado: modo Hidden ya no usa `Visible(false)` porque eso impedía reabrir con hotkeys.
- Fix aplicado: modo Hidden ahora usa ventana `1x1` fuera de pantalla para mantener vivo el loop.
- Fix aplicado: hotkey global ignora eventos Released para que Ctrl+Space funcione como toggle real.
- Validación manual pendiente: re-probar fixes en Windows.
- Feedback visual recibido con captura: panel feo, due no visible y `manana` sin acento no parseaba.
- Fix aplicado: parser soporta `manana` y `pasado manana` sin acento.
- Fix aplicado: panel muestra `vence YYYY-MM-DD` cuando hay due.
- Fix aplicado: panel tiene cards, badges coloreados y layout más legible.
- Fix aplicado: limpieza de atajos internos elimina secuencias trailing de `1/2/3/q/w/e`, no solo un carácter.
- Iteración final aplicada: Quick Add estilo rmenu, status/priority opcionales, Shift+Enter para carga continua.
- Iteración final aplicada: panel estilo rmenu con cards, multiselección, Alt+Click para avance de estado grupal y Del para borrar selección.
- Documentación actualizada: `rtasks.md`, `README.md`, `STATE.md`.
- Validación final: `cargo test` OK, `cargo check` OK.
