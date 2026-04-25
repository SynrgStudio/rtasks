RTasks — Spec completo
1. Objetivo

Una herramienta desktop para Windows que haga solo 2 cosas:

Capturar tareas en menos de 2 segundos.
Mostrar las tareas activas en un panel mínimo.

No es Notion. No es TickTick. No es calendario. No es GTD completo.

Es:

hotkey → escribo → Enter → tarea guardada

2. UX principal
Modal de captura

Hotkey:

Alt + Spacebar

Abre un modal centrado, ultra rápido.

Contenido visual:

comprar pan          TODO   prio:media

Atajos dentro del modal:

Alt + 1 → TODO
Alt + 2 → DOING
Alt + 3 → DONE

Alt + Q → prio:alta
Alt + W → prio:media
Alt + E → prio:baja

Enter       → guardar y cerrar
Shift+Enter → guardar, limpiar y mantener abierto para carga continua
Esc         → cerrar sin guardar

Defaults visuales:

status: TODO (placeholder gris, no se guarda si no se activa)
priority: media (placeholder gris, no se guarda si no se activa)

Regla de toggle:

Alt + 1 activa/desactiva TODO
Alt + 2 activa/desactiva DOING
Alt + 3 activa/desactiva DONE
Alt + Q activa/desactiva prio:alta
Alt + W activa/desactiva prio:media
Alt + E activa/desactiva prio:baja

Ejemplos:

comprar pan
comprar pan mañana
pagar dominio el viernes
terminé backup alt+3
Panel de tareas

Hotkey:

Ctrl + Spacebar

Abre/cierra panel en la esquina inferior derecha.

Muestra solo:

TODO
DOING

No muestra DONE, salvo modo debug/filtro futuro.

Visual:

RTasks

TODO
[ALTA] pagar mercadopago
[MEDIA] comprar pan
[BAJA] ordenar escritorio

DOING
[ALTA] reparar STL viewer
[MEDIA] escribir spec

Acciones mínimas en panel:

Esc → cerrar y limpiar selección
Click tarea → seleccionar/deseleccionar tarea
Alt + Click → avanzar estado
  - TODO/sin estado → DOING
  - DOING → DONE
  - DONE → TODO si se vuelve a mostrar en modo futuro
Alt + Click sobre una tarea seleccionada → avanzar toda la selección
Del → borrar tareas seleccionadas

Nada más por ahora.

3. Estructura exacta de datos
Task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,

    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,

    pub due: Option<String>,

    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,

    pub source: TaskSource,
}
Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Todo,
    Doing,
    Done,
}
Priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
}

Orden real para display:

High → Medium → Low
Source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskSource {
    DesktopQuickAdd,
    DesktopPanel,
    Import,
}
4. Formato de guardado
Opción recomendada para V1: JSONL

No SQLite todavía.

Archivo:

%APPDATA%\rtasks\tasks.jsonl

Cada línea es una tarea completa.

Ejemplo:

{"id":"01HZY5E6WHK9Q1M8Z7K3D7X4D2","title":"comprar pan","status":"Todo","priority":"Medium","due":"2026-04-26","created_at":"2026-04-25T18:30:00-03:00","updated_at":"2026-04-25T18:30:00-03:00","completed_at":null,"source":"DesktopQuickAdd"}
{"id":"01HZY5FDNG1R59C6V3KH8P1N7A","title":"reparar STL viewer","status":"Doing","priority":"High","due":null,"created_at":"2026-04-25T18:31:00-03:00","updated_at":"2026-04-25T18:31:00-03:00","completed_at":null,"source":"DesktopQuickAdd"}
Por qué JSONL y no SQLite en V1

Porque es:

simple
debuggeable
portable
append-friendly
fácil de reparar
fácil de exportar

No necesitás DB hasta que tengas:

+1000 tareas
sync
queries complejas
historial real

Y no estamos haciendo eso.

5. Escritura segura

No editar línea por línea.

Para guardar cambios:

cargar todo en memoria
modificar
escribir tasks.tmp
renombrar a tasks.jsonl

Esto evita corrupción si el proceso muere.

tasks.jsonl
tasks.tmp
6. Orden de visualización

Regla:

status ASC
priority DESC
created_at ASC

Display:

TODO primero
DOING después

Dentro de cada estado:
High
Medium
Low

Dentro de prioridad:
más viejas primero

Porque una tarea vieja de alta prioridad no debería quedar enterrada.

7. Parser mínimo de lenguaje natural

V1 soporta solo esto:

mañana
hoy
pasado mañana

Opcional:

el lunes
el martes
el miércoles
...

Ejemplo:

Input:

comprar pan mañana

Resultado:

{
  "title": "comprar pan",
  "due": "2026-04-26"
}

La palabra temporal se remueve del título.

No metas IA. No metas LLM. No metas chrono avanzado todavía.

8. Stack Rust recomendado
Opción más práctica
Rust
eframe / egui
global-hotkey
serde
serde_json
chrono
directories
ulid

egui te da UI inmediata, rápida, simple, single exe-friendly.

Tauri es demasiado para esto.

Iced es lindo, pero más pesado para overlay/hotkey/ventanas chicas.

Para este caso:

egui gana por velocidad de implementación y cero ceremonia.

9. Crates
[dependencies]
eframe = "0.27"
egui = "0.27"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
directories = "5"
ulid = "1"
global-hotkey = "0.5"
anyhow = "1"

Puede variar versión, pero esa familia es la idea.

10. Arquitectura
src/
  main.rs
  app.rs
  task.rs
  storage.rs
  parser.rs
  hotkeys.rs
  ui/
    quick_add.rs
    panel.rs

Si querés mantenerlo más compacto:

src/
  main.rs
  task.rs
  storage.rs
  parser.rs

Para V1, incluso un solo main.rs es aceptable.

11. Estado interno
pub struct RTasksApp {
    pub mode: AppMode,
    pub tasks: Vec<Task>,

    pub quick_input: String,
    pub selected_status: TaskStatus,
    pub selected_priority: Priority,

    pub selected_task_index: Option<usize>,
}
pub enum AppMode {
    Hidden,
    QuickAdd,
    Panel,
}
12. Comportamiento de ventanas

Idealmente tenés una sola app con dos ventanas:

Quick Add Window
sin borde
always on top
centrada
ancho: 720px
alto: 120px
focus automático en input
Panel Window
sin borde o borde mínimo
always on top
bottom-right
ancho: 420px
alto: 520px
transparencia leve opcional

Regla importante:

cerrar ventana ≠ cerrar app

El exe queda corriendo en background/tray.

13. System tray

V1 incluye tray mínimo:

RTasks
- Open Quick Add
- Open Panel
- Quit

El ícono se genera en runtime con una `R` simple.

14. Implementation Plan
Fase 0 — Skeleton

Objetivo: exe abre ventana vacía.

cargo new rtasks
agregar eframe
mostrar ventana egui básica

Done cuando:

cargo run
abre ventana
cierra sin errores
Fase 1 — Modelo + storage

Implementar:

Task
TaskStatus
Priority
load_tasks()
save_tasks()
add_task()
update_task()

Archivo:

%APPDATA%\rtasks\tasks.jsonl

Done cuando:

cargo run
crea archivo
guarda tarea hardcodeada
la vuelve a cargar
Fase 2 — Quick Add UI

Implementar ventana con:

input
status visible opcional
priority visible opcional
Enter
Shift+Enter
Esc
Alt+1/2/3 toggle status
Alt+Q/W/E toggle priority

Done cuando:

escribís "comprar pan"
Enter
queda guardado
input se limpia
ventana se oculta

Shift+Enter guarda, limpia y mantiene el Quick Add abierto
Fase 3 — Parser mínimo

Implementar:

mañana
hoy
pasado mañana

Done cuando:

"comprar pan mañana"

se guarda como:

title: comprar pan
due: fecha de mañana
Fase 4 — Panel UI

Implementar:

secciones TODO / DOING
orden por prioridad
click selecciona/deselecciona
multiselección sin modificadores
Alt+Click avanza estado
Alt+Click en tarea seleccionada avanza toda la selección
Del borra seleccionadas
Esc cierra y limpia selección

Done cuando:

Ctrl+Spacebar
abre panel
ves tareas activas
podés seleccionar, avanzar estado y borrar tareas
Fase 5 — Hotkeys globales

Implementar:

Alt+Spacebar → QuickAdd
Ctrl+Spacebar → Panel

Done cuando:

funciona desde cualquier app

Este es el punto donde ya tenés producto usable.

Fase 6 — Polish mínimo

Solo:

always on top
posición correcta
focus automático
tema dark
fuente linda
tray mínimo
script local de release/package
animación NO
settings NO
cloud NO
tags NO
14.1 Tasklist de implementación

- [x] Crear proyecto Rust `rtasks`.
- [x] Agregar dependencias base: `eframe`, `egui`, `serde`, `serde_json`, `chrono`, `directories`, `ulid`, `global-hotkey`, `anyhow`.
- [x] Crear estructura de archivos: `main.rs`, `app.rs`, `task.rs`, `storage.rs`, `parser.rs`, `hotkeys.rs`, `ui/quick_add.rs`, `ui/panel.rs`.
- [x] Implementar `Task` con `id`, `title`, `status`, `priority`, `due`, `created_at`, `updated_at`, `completed_at`, `source` (`status` y `priority` son opcionales en la implementación actual).
- [x] Implementar enums `TaskStatus`, `Priority`, `TaskSource` con `Serialize` y `Deserialize`.
- [x] Implementar orden de display: `Todo` antes que `Doing`, prioridad `High` → `Medium` → `Low`, `created_at` ascendente.
- [x] Resolver ruta de datos `%APPDATA%\rtasks\tasks.jsonl`.
- [x] Crear carpeta `%APPDATA%\rtasks\` si no existe.
- [x] Implementar `load_tasks()` leyendo JSONL.
- [x] Implementar tolerancia básica a líneas vacías en JSONL.
- [x] Implementar `save_tasks()` con escritura segura: escribir `tasks.tmp` y renombrar a `tasks.jsonl`.
- [x] Implementar `add_task()`.
- [x] Implementar `update_task()`.
- [x] Implementar borrado de tarea desde memoria y persistencia posterior.
- [x] Crear estado interno `RTasksApp` con `mode`, `tasks`, `quick_input`, `selected_status`, `selected_priority`, multiselección de tareas y estado de hotkeys.
- [x] Crear `AppMode`: `Hidden`, `QuickAdd`, `Panel`.
- [x] Implementar ventana base con `eframe/egui`.
- [x] Implementar tema dark mínimo.
- [x] Implementar modal Quick Add centrado, always on top, ancho aproximado 720px y alto aproximado 120px.
- [x] Implementar foco automático en input al abrir Quick Add.
- [x] Implementar input de texto para captura.
- [x] Mostrar status seleccionado en Quick Add.
- [x] Mostrar prioridad seleccionada en Quick Add.
- [x] Implementar placeholders visuales: status `Todo`, priority `Medium`, sin guardar propiedades salvo activación explícita.
- [x] Implementar `Enter` en Quick Add para guardar tarea y cerrar.
- [x] Implementar `Esc` en Quick Add para cerrar sin guardar.
- [x] Implementar `Alt+1` para status `Todo`.
- [x] Implementar `Alt+2` para status `Doing`.
- [x] Implementar `Alt+3` para status `Done`.
- [x] Implementar `Alt+Q` para prioridad `High`.
- [x] Implementar `Alt+W` para prioridad `Medium`.
- [x] Implementar `Alt+E` para prioridad `Low`.
- [x] Implementar `Shift+Enter` en Quick Add para guardar, limpiar y mantener abierto.
- [x] Limpiar input después de guardar.
- [x] Ocultar Quick Add después de guardar con Enter.
- [x] Implementar parser mínimo para `hoy`.
- [x] Implementar parser mínimo para `mañana`.
- [x] Implementar parser mínimo para `pasado mañana`.
- [x] Remover palabra temporal detectada del título.
- [x] Guardar `due` como fecha ISO `YYYY-MM-DD` cuando aplique.
- [x] Implementar panel bottom-right, always on top, ancho aproximado 420px y alto aproximado 520px.
- [x] Mostrar título `RTasks` en panel.
- [x] Mostrar sección `TODO`.
- [x] Mostrar sección `DOING`.
- [x] Ocultar tareas `DONE` en panel.
- [x] Renderizar tareas con etiqueta de prioridad `[ALTA]`, `[MEDIA]`, `[BAJA]`.
- [x] Aplicar orden de visualización en panel.
- [x] Implementar selección de tarea en panel.
- [x] Implementar click en tarea para seleccionar/deseleccionar.
- [x] Implementar multiselección sin modificadores.
- [x] Implementar `Alt+Click` para avanzar estado.
- [x] Implementar `Alt+Click` sobre tarea seleccionada para avanzar toda la selección.
- [x] Implementar `Del` para borrar tareas seleccionadas.
- [x] Implementar `Esc` en panel para cerrar.
- [x] Registrar hotkey global `Alt+Spacebar` para abrir Quick Add.
- [x] Registrar hotkey global `Ctrl+Spacebar` para abrir/cerrar panel.
- [x] Asegurar que cerrar ventana no cierre la app.
- [x] Mantener proceso corriendo en background mientras hotkeys estén activas.
- [x] Implementar tray mínimo: Open Quick Add, Open Panel, Quit.
- [x] Implementar script `release-local.ps1` para build release y paquete zip local.
- [x] Validar que Quick Add funciona desde otra app.
- [x] Validar que Panel funciona desde otra app.
- [x] Validar que una tarea guardada persiste después de cerrar y abrir la app.
- [x] Validar que cambios de estado se persisten en `tasks.jsonl`.
- [x] Validar que borrar tarea actualiza `tasks.jsonl`.
- [x] Validar manualmente el flujo MVP completo durante uso real.

15. MVP real

El MVP es esto:

Alt+Spacebar
escribir tarea
Alt+Q/W/E opcional y toggleable
Alt+1/2/3 opcional y toggleable
Enter para guardar y cerrar
Shift+Enter para guardar y seguir cargando
Ctrl+Spacebar
ver tareas
click para seleccionar/deseleccionar
Alt+Click para avanzar estado
Del para borrar selección

Eso ya es suficiente.

No agregues más hasta usarlo 7 días.

16. Cosas prohibidas en V1

Prohibidísimo:

sync
login
Android
notificaciones
recurrencias
subtareas
proyectos
tags custom
calendario visual
integraciones
Markdown
rich text
plugin system
temas configurables
IA
LLM

Especialmente plugin system. Te conozco.

17. Roadmap sano
V1
desktop local
JSONL
quick add
panel
hotkeys
tray mínimo
release zip local
parser mínimo
V1.1
editar título
filtro por due hoy/mañana
exportar JSON
V1.2
SQLite opcional
backup automático
V2
Android widget
sync local por carpeta
V3, si realmente vale la pena
producto público
landing
installer
autoupdate
18. Decisión técnica clave

Mi recomendación final:

Rust + egui + JSONL

No Tauri.
No webview.
No backend.
No Supabase.
No Anytype al principio.

Hacelo local, rápido, idiota, confiable.

19. Nombre de archivo/config
%APPDATA%\rtasks\
  tasks.jsonl
  config.json
  backup\

config.json mínimo:

{
  "quick_add_hotkey": "Alt+Spacebar",
  "panel_hotkey": "Ctrl+Spacebar",
  "default_status": "Todo",
  "default_priority": "Medium"
}

Pero V1 puede hardcodearlo.

20. Regla madre

Cada feature nueva tiene que pasar esta pregunta:

¿Esto reduce fricción o me está dando una excusa para jugar a diseñar TickTick?

Si no reduce fricción, no entra.

El producto es bueno justamente porque es brutalmente chico.