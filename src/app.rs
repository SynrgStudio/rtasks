use chrono::{SecondsFormat, Utc};
use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;

use eframe::egui;
use ulid::Ulid;
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

use crate::{
    hotkeys::{HotkeyAction, Hotkeys},
    ipc::{self, IpcCommand, IpcRequest},
    parser::parse_task_input,
    storage,
    task::{Priority, Task, TaskSource, TaskStatus},
    tray::{AppTray, TrayAction},
    ui::{panel, quick_add},
};

pub struct RTasksApp {
    pub mode: AppMode,
    pub tasks: Vec<Task>,
    pub quick_input: String,
    pub selected_status: Option<TaskStatus>,
    pub selected_priority: Option<Priority>,
    pub selected_task_indices: Vec<usize>,
    pub last_error: Option<String>,
    pub suppress_ctrl_click: bool,
    hotkeys: Option<Hotkeys>,
    tray: Option<AppTray>,
    ipc_receiver: Option<Receiver<IpcRequest>>,
    _ipc_thread: Option<JoinHandle<()>>,
    applied_mode: Option<AppMode>,
    quit_requested: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Hidden,
    QuickAdd,
    Panel,
}

impl RTasksApp {
    pub fn new() -> Self {
        Self::new_with_mode(AppMode::QuickAdd)
    }

    pub fn new_with_mode(mode: AppMode) -> Self {
        Self::new_with_mode_and_hotkeys(mode, true)
    }

    pub fn new_with_mode_and_hotkeys(mode: AppMode, enable_hotkeys: bool) -> Self {
        let (tasks, last_error) = match storage::load_tasks() {
            Ok(tasks) => (tasks, None),
            Err(error) => (Vec::new(), Some(error.to_string())),
        };

        let (hotkeys, hotkey_error) = if enable_hotkeys {
            match Hotkeys::register() {
                Ok(hotkeys) => (Some(hotkeys), None),
                Err(error) => (None, Some(error.to_string())),
            }
        } else {
            (None, None)
        };

        let (tray, tray_error) = if enable_hotkeys {
            match AppTray::new() {
                Ok(tray) => (Some(tray), None),
                Err(error) => (None, Some(error.to_string())),
            }
        } else {
            (None, None)
        };

        let (ipc_sender, ipc_receiver) = mpsc::channel();
        let ipc_thread = ipc::start_named_pipe_server(ipc_sender);

        Self {
            mode,
            tasks,
            quick_input: String::new(),
            selected_status: None,
            selected_priority: None,
            selected_task_indices: Vec::new(),
            last_error: last_error.or(hotkey_error).or(tray_error),
            suppress_ctrl_click: false,
            hotkeys,
            tray,
            ipc_receiver: Some(ipc_receiver),
            _ipc_thread: Some(ipc_thread),
            applied_mode: None,
            quit_requested: false,
        }
    }

    pub fn save_quick_input(&mut self) {
        self.save_quick_input_with_close(true);
    }

    pub fn save_quick_input_and_continue(&mut self) {
        self.save_quick_input_with_close(false);
    }

    fn save_quick_input_with_close(&mut self, close_after_save: bool) {
        let parsed = parse_task_input(&self.quick_input);
        let title = parsed.title.trim();

        if title.is_empty() {
            if close_after_save {
                self.close_quick_add();
            } else {
                self.reset_quick_add_fields();
            }
            return;
        }

        let now = now_timestamp();
        let task = Task {
            id: Ulid::new().to_string(),
            title: title.to_owned(),
            status: self.selected_status.clone(),
            priority: self.selected_priority.clone(),
            due: parsed.due,
            created_at: now.clone(),
            updated_at: now,
            completed_at: None,
            source: TaskSource::DesktopQuickAdd,
        };

        match storage::add_task(&mut self.tasks, task) {
            Ok(()) => self.last_error = None,
            Err(error) => self.last_error = Some(error.to_string()),
        }

        self.reset_quick_add_fields();
        if close_after_save {
            self.mode = AppMode::Hidden;
        } else {
            self.mode = AppMode::QuickAdd;
        }
    }

    fn reset_quick_add_fields(&mut self) {
        self.quick_input.clear();
        self.selected_status = None;
        self.selected_priority = None;
    }

    pub fn close_quick_add(&mut self) {
        self.reset_quick_add_fields();
        self.mode = AppMode::Hidden;
    }

    pub fn persist_tasks(&mut self) {
        match storage::save_tasks(&self.tasks) {
            Ok(()) => self.last_error = None,
            Err(error) => self.last_error = Some(error.to_string()),
        }
    }

    pub fn active_task_indices_sorted(&self) -> Vec<usize> {
        let mut indices = self
            .tasks
            .iter()
            .enumerate()
            .filter_map(|(index, task)| match task.status {
                Some(TaskStatus::Done) => None,
                None | Some(TaskStatus::Todo) | Some(TaskStatus::Doing) => Some(index),
            })
            .collect::<Vec<_>>();

        indices.sort_by(|left, right| {
            let left_task = &self.tasks[*left];
            let right_task = &self.tasks[*right];

            status_rank(left_task.status.as_ref())
                .cmp(&status_rank(right_task.status.as_ref()))
                .then_with(|| {
                    priority_rank(right_task.priority.as_ref())
                        .cmp(&priority_rank(left_task.priority.as_ref()))
                })
                .then_with(|| left_task.created_at.cmp(&right_task.created_at))
        });

        indices
    }

    pub fn task_effective_status(&self, index: usize) -> Option<&TaskStatus> {
        self.tasks.get(index).and_then(|task| task.status.as_ref())
    }

    pub fn toggle_task_status(&mut self, index: usize) {
        self.advance_task_statuses(&[index]);
    }

    pub fn advance_selected_or_task(&mut self, index: usize) {
        if self.is_task_selected(index) && self.selected_task_indices.len() > 1 {
            let indices = self.selected_task_indices.clone();
            self.advance_task_statuses(&indices);
        } else {
            self.advance_task_statuses(&[index]);
        }
        self.clear_task_selection();
    }

    fn advance_task_statuses(&mut self, indices: &[usize]) {
        let now = now_timestamp();
        let mut sorted_indices = indices.to_vec();
        sorted_indices.sort_unstable();
        sorted_indices.dedup();

        for index in sorted_indices {
            let Some(task) = self.tasks.get_mut(index) else {
                continue;
            };

            match task.status {
                None | Some(TaskStatus::Todo) => {
                    task.status = Some(TaskStatus::Doing);
                    task.completed_at = None;
                }
                Some(TaskStatus::Doing) => {
                    task.status = Some(TaskStatus::Done);
                    task.completed_at = Some(now.clone());
                }
                Some(TaskStatus::Done) => {
                    task.status = Some(TaskStatus::Todo);
                    task.completed_at = None;
                }
            }
            task.updated_at = now.clone();
        }

        self.persist_tasks();
    }

    pub fn complete_task(&mut self, index: usize) {
        let Some(task) = self.tasks.get_mut(index) else {
            return;
        };

        let now = now_timestamp();
        task.status = Some(TaskStatus::Done);
        task.updated_at = now.clone();
        task.completed_at = Some(now);
        self.persist_tasks();
    }

    pub fn open_quick_add(&mut self) {
        self.mode = AppMode::QuickAdd;
    }

    pub fn save_ipc_task(&mut self, request: IpcRequest) {
        let input = request.input.unwrap_or_default();
        let parsed = parse_task_input(&input);
        let title = parsed.title.trim();
        if title.is_empty() {
            return;
        }

        let now = now_timestamp();
        let task = Task {
            id: Ulid::new().to_string(),
            title: title.to_owned(),
            status: request.status,
            priority: request.priority,
            due: parsed.due,
            created_at: now.clone(),
            updated_at: now,
            completed_at: None,
            source: TaskSource::DesktopQuickAdd,
        };

        match storage::add_task(&mut self.tasks, task) {
            Ok(()) => self.last_error = None,
            Err(error) => self.last_error = Some(error.to_string()),
        }
    }

    pub fn toggle_panel(&mut self) {
        self.mode = if self.mode == AppMode::Panel {
            self.suppress_ctrl_click = false;
            AppMode::Hidden
        } else {
            self.suppress_ctrl_click = true;
            AppMode::Panel
        };
    }

    pub fn update_ctrl_suppression(&mut self, ctx: &egui::Context) {
        let ctrl_down = ctx.input(|input| input.modifiers.ctrl);
        if !ctrl_down {
            self.suppress_ctrl_click = false;
        }
    }

    pub fn ctrl_click_active(&self, ctx: &egui::Context) -> bool {
        ctx.input(|input| input.modifiers.ctrl) && !self.suppress_ctrl_click
    }

    pub fn handle_ipc_commands(&mut self, ctx: &egui::Context) {
        let Some(receiver) = &self.ipc_receiver else {
            return;
        };

        let commands = receiver.try_iter().collect::<Vec<_>>();
        for request in commands {
            match request.cmd {
                IpcCommand::QuickAdd => self.open_quick_add(),
                IpcCommand::Panel => self.toggle_panel(),
                IpcCommand::AddTask => self.save_ipc_task(request),
                IpcCommand::Shutdown => {
                    self.quit_requested = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    pub fn handle_hotkeys(&mut self) {
        let Some(hotkeys) = &self.hotkeys else {
            return;
        };

        for action in hotkeys.drain_actions() {
            match action {
                HotkeyAction::OpenQuickAdd => self.open_quick_add(),
                HotkeyAction::TogglePanel => self.toggle_panel(),
            }
        }
    }

    pub fn handle_tray_actions(&mut self, ctx: &egui::Context) {
        let Some(tray) = &self.tray else {
            return;
        };

        for action in tray.drain_actions() {
            match action {
                TrayAction::OpenQuickAdd => self.open_quick_add(),
                TrayAction::OpenPanel => {
                    self.mode = AppMode::Panel;
                    self.suppress_ctrl_click = false;
                }
                TrayAction::Quit => {
                    self.quit_requested = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    pub fn handle_close_request(&mut self, ctx: &egui::Context) {
        let close_requested = ctx.input(|input| input.viewport().close_requested());
        if close_requested && !self.quit_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.mode = AppMode::Hidden;
        }
    }

    pub fn apply_viewport_for_mode(&mut self, ctx: &egui::Context) {
        if self.applied_mode == Some(self.mode) {
            return;
        }
        self.applied_mode = Some(self.mode);

        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
            egui::WindowLevel::AlwaysOnTop,
        ));
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));

        let size = match self.mode {
            AppMode::Hidden => egui::vec2(1.0, 1.0),
            AppMode::QuickAdd => egui::vec2(760.0, 72.0),
            AppMode::Panel => egui::vec2(420.0, 520.0),
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));

        if self.mode != AppMode::Hidden {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        if let Some(position) = viewport_position(ctx, self.mode, size) {
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(position));
        }
    }

    pub fn delete_selected_tasks(&mut self) {
        if self.selected_task_indices.is_empty() {
            return;
        }

        let mut indices = self.selected_task_indices.clone();
        indices.sort_unstable();
        indices.dedup();

        for index in indices.into_iter().rev() {
            if index < self.tasks.len() {
                self.tasks.remove(index);
            }
        }

        self.selected_task_indices.clear();
        self.persist_tasks();
    }

    pub fn toggle_task_selection(&mut self, index: usize) {
        if let Some(position) = self
            .selected_task_indices
            .iter()
            .position(|selected_index| *selected_index == index)
        {
            self.selected_task_indices.remove(position);
        } else {
            self.selected_task_indices.push(index);
        }
    }

    pub fn clear_task_selection(&mut self) {
        self.selected_task_indices.clear();
    }

    pub fn is_task_selected(&self, index: usize) -> bool {
        self.selected_task_indices.contains(&index)
    }
}

fn viewport_position(ctx: &egui::Context, mode: AppMode, size: egui::Vec2) -> Option<egui::Pos2> {
    let viewport = ctx.input(|input| input.viewport().clone());
    let monitor_size = viewport.monitor_size.unwrap_or_else(primary_screen_size);
    let side_margin = 24.0;
    let bottom_margin = 72.0;

    match mode {
        AppMode::Hidden => Some(egui::pos2(-10_000.0, -10_000.0)),
        AppMode::QuickAdd => Some(egui::pos2(
            (monitor_size.x - size.x) * 0.5,
            (monitor_size.y - size.y) * 0.35,
        )),
        AppMode::Panel => Some(egui::pos2(
            monitor_size.x - size.x - side_margin,
            monitor_size.y - size.y - bottom_margin,
        )),
    }
}

fn primary_screen_size() -> egui::Vec2 {
    unsafe {
        egui::vec2(
            GetSystemMetrics(SM_CXSCREEN) as f32,
            GetSystemMetrics(SM_CYSCREEN) as f32,
        )
    }
}

fn status_rank(status: Option<&TaskStatus>) -> u8 {
    match status {
        None | Some(TaskStatus::Todo) => 0,
        Some(TaskStatus::Doing) => 1,
        Some(TaskStatus::Done) => 2,
    }
}

fn priority_rank(priority: Option<&Priority>) -> u8 {
    match priority {
        Some(Priority::High) => 2,
        None | Some(Priority::Medium) => 1,
        Some(Priority::Low) => 0,
    }
}

impl Default for RTasksApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for RTasksApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        self.handle_ipc_commands(ctx);
        self.handle_hotkeys();
        self.handle_tray_actions(ctx);
        self.handle_close_request(ctx);
        self.apply_viewport_for_mode(ctx);
        ctx.request_repaint_after(std::time::Duration::from_millis(50));

        match self.mode {
            AppMode::Hidden => {}
            AppMode::QuickAdd => quick_add::show(ctx, self),
            AppMode::Panel => panel::show(ctx, self),
        }
    }
}

fn now_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}
