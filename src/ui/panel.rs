use eframe::egui::{self, Color32, FontFamily, FontId, Frame, Key, Margin, RichText, Stroke, TextStyle};

use crate::{
    app::{AppMode, RTasksApp},
    task::{Priority, TaskStatus},
};

const BACKGROUND: Color32 = Color32::from_rgb(40, 44, 52);
const FOREGROUND: Color32 = Color32::from_rgb(171, 178, 191);
const SELECTED_BACKGROUND: Color32 = Color32::from_rgb(58, 63, 75);
const SELECTED_FOREGROUND: Color32 = Color32::from_rgb(230, 230, 230);
const BORDER: Color32 = Color32::from_rgb(33, 37, 43);
const MUTED: Color32 = Color32::from_rgb(122, 128, 140);
const ROW_BACKGROUND: Color32 = Color32::from_rgb(46, 51, 61);
const ROW_BORDER: Color32 = Color32::from_rgb(58, 65, 78);
const HIGH: Color32 = Color32::from_rgb(224, 92, 92);
const MEDIUM: Color32 = Color32::from_rgb(224, 172, 74);
const LOW: Color32 = Color32::from_rgb(107, 171, 119);
const TEXT_SIZE: f32 = 15.0;
const HINT_SIZE: f32 = 13.0;

pub fn show(ctx: &egui::Context, app: &mut RTasksApp) {
    apply_panel_style(ctx);
    handle_keyboard(ctx, app);

    let mut clicked_task_index = None;

    egui::CentralPanel::default()
        .frame(Frame::none().fill(BACKGROUND))
        .show(ctx, |ui| {
            Frame::none()
                .fill(BACKGROUND)
                .stroke(Stroke::new(1.0, BORDER))
                .inner_margin(Margin::same(0.0))
                .show(ui, |ui| {
                    show_header(ui);
                    show_section(ui, app, TaskStatus::Todo, "TODO", &mut clicked_task_index);
                    show_section(ui, app, TaskStatus::Doing, "DOING", &mut clicked_task_index);

                    if let Some(error) = &app.last_error {
                        ui.add_space(4.0);
                        ui.colored_label(Color32::RED, error);
                    }
                });
        });

    if let Some(index) = clicked_task_index {
        app.advance_selected_or_task(index);
    }
}

fn show_header(ui: &mut egui::Ui) {
    Frame::none()
        .fill(SELECTED_BACKGROUND)
        .inner_margin(Margin::symmetric(8.0, 7.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("RTasks")
                        .font(mono_font(TEXT_SIZE))
                        .color(SELECTED_FOREGROUND),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Esc cerrar")
                            .font(mono_font(HINT_SIZE))
                            .color(SELECTED_FOREGROUND),
                    );
                });
            });
        });

    Frame::none()
        .fill(BACKGROUND)
        .inner_margin(Margin::symmetric(8.0, 7.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new("Click selecciona · Alt+Click avanza · Del borra seleccionada")
                    .font(mono_font(HINT_SIZE))
                    .color(MUTED),
            );
        });
}

fn show_section(
    ui: &mut egui::Ui,
    app: &mut RTasksApp,
    status: TaskStatus,
    title: &str,
    clicked_task_index: &mut Option<usize>,
) {
    Frame::none()
        .fill(BACKGROUND)
        .inner_margin(Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            let indices = app.active_task_indices_sorted();
            let visible_indices = indices
                .into_iter()
                .filter(|index| effective_status(app.tasks[*index].status.as_ref()) == status)
                .collect::<Vec<_>>();

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(title)
                        .font(mono_font(HINT_SIZE))
                        .color(SELECTED_FOREGROUND),
                );
                ui.label(
                    RichText::new(visible_indices.len().to_string())
                        .font(mono_font(HINT_SIZE))
                        .color(MUTED),
                );
            });

            if visible_indices.is_empty() {
                ui.add_space(2.0);
                ui.label(RichText::new("Sin tareas").font(mono_font(HINT_SIZE)).color(MUTED));
                return;
            }

            ui.add_space(4.0);
            for index in visible_indices {
                show_task_row(ui, app, index, clicked_task_index);
                ui.add_space(6.0);
            }
        });
}

fn show_task_row(
    ui: &mut egui::Ui,
    app: &mut RTasksApp,
    index: usize,
    clicked_task_index: &mut Option<usize>,
) {
    let selected = app.is_task_selected(index);
    let fill = if selected { SELECTED_BACKGROUND } else { ROW_BACKGROUND };
    let text_color = if selected { SELECTED_FOREGROUND } else { FOREGROUND };
    let border_color = if selected { SELECTED_FOREGROUND } else { ROW_BORDER };

    let frame_output = Frame::none()
        .fill(fill)
        .stroke(Stroke::new(1.0, border_color))
        .rounding(4.0)
        .inner_margin(Margin::symmetric(9.0, 7.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(status_checkbox(app.tasks[index].status.as_ref()))
                        .font(mono_font(TEXT_SIZE))
                        .color(status_color(app.tasks[index].status.as_ref())),
                );

                let priority_width = 58.0;
                ui.allocate_ui_with_layout(
                    egui::vec2(priority_width, 20.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        if let Some(priority) = &app.tasks[index].priority {
                            ui.label(
                                RichText::new(priority_label(priority))
                                    .font(mono_font(HINT_SIZE))
                                    .strong()
                                    .color(priority_color(priority)),
                            );
                        } else {
                            ui.label(RichText::new("-").font(mono_font(HINT_SIZE)).color(MUTED));
                        }
                    },
                );

                let due_width = 112.0;
                let title_width = (ui.available_width() - due_width).max(120.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(title_width, 20.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.label(
                            RichText::new(&app.tasks[index].title)
                                .font(mono_font(TEXT_SIZE))
                                .color(text_color),
                        );
                    },
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(due) = &app.tasks[index].due {
                        ui.label(
                            RichText::new(format!("vence {due}"))
                                .font(mono_font(HINT_SIZE))
                                .color(if selected { SELECTED_FOREGROUND } else { MUTED }),
                        );
                    }
                });
            });
        });
    let response = ui.interact(
        frame_output.response.rect,
        ui.id().with(("task-row", index)),
        egui::Sense::click(),
    );

    if response.clicked() {
        if ui.input(|input| input.modifiers.alt) {
            *clicked_task_index = Some(index);
        } else {
            app.toggle_task_selection(index);
        }
    }
}

fn handle_keyboard(ctx: &egui::Context, app: &mut RTasksApp) {
    let mut close = false;
    let mut delete = false;

    ctx.input(|input| {
        if input.key_pressed(Key::Escape) {
            close = true;
        }
        if input.key_pressed(Key::Delete) {
            delete = true;
        }
    });

    if delete {
        app.delete_selected_tasks();
    }

    if close {
        app.clear_task_selection();
        app.mode = AppMode::Hidden;
    }
}

fn apply_panel_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = BACKGROUND;
    style.visuals.window_fill = BACKGROUND;
    style.visuals.extreme_bg_color = BACKGROUND;
    style.visuals.widgets.inactive.bg_fill = BACKGROUND;
    style.visuals.widgets.inactive.fg_stroke.color = FOREGROUND;
    style.visuals.widgets.active.fg_stroke.color = SELECTED_FOREGROUND;
    style.visuals.widgets.hovered.fg_stroke.color = SELECTED_FOREGROUND;
    style.text_styles.insert(TextStyle::Body, mono_font(TEXT_SIZE));
    style
        .text_styles
        .insert(TextStyle::Monospace, mono_font(TEXT_SIZE));
    ctx.set_style(style);
}

fn effective_status(status: Option<&TaskStatus>) -> TaskStatus {
    match status {
        Some(TaskStatus::Doing) => TaskStatus::Doing,
        Some(TaskStatus::Done) => TaskStatus::Done,
        None | Some(TaskStatus::Todo) => TaskStatus::Todo,
    }
}

fn status_checkbox(status: Option<&TaskStatus>) -> &'static str {
    match status {
        Some(TaskStatus::Doing) => "[>]",
        Some(TaskStatus::Done) => "[x]",
        None | Some(TaskStatus::Todo) => "[ ]",
    }
}

fn status_color(status: Option<&TaskStatus>) -> Color32 {
    match status {
        Some(TaskStatus::Doing) => MEDIUM,
        Some(TaskStatus::Done) => LOW,
        None | Some(TaskStatus::Todo) => MUTED,
    }
}

fn priority_label(priority: &Priority) -> &'static str {
    match priority {
        Priority::High => "ALTA",
        Priority::Medium => "MEDIA",
        Priority::Low => "BAJA",
    }
}

fn priority_color(priority: &Priority) -> Color32 {
    match priority {
        Priority::High => HIGH,
        Priority::Medium => MEDIUM,
        Priority::Low => LOW,
    }
}

fn mono_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

pub fn open(app: &mut RTasksApp) {
    app.mode = AppMode::Panel;
}
