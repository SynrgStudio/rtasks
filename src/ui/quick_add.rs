use eframe::egui::{
    self, Color32, FontFamily, FontId, Frame, Key, Margin, RichText, Stroke, TextStyle,
};

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
const TEXT_SIZE: f32 = 15.0;
const HINT_SIZE: f32 = 13.0;

pub fn show(ctx: &egui::Context, app: &mut RTasksApp) {
    apply_command_palette_style(ctx);

    egui::CentralPanel::default()
        .frame(Frame::none().fill(BACKGROUND))
        .show(ctx, |ui| {
            Frame::none()
                .fill(BACKGROUND)
                .stroke(Stroke::new(1.0, BORDER))
                .inner_margin(Margin::same(0.0))
                .show(ui, |ui| {
                    show_input_row(ui, app);
                    show_hint_row(ui, app);
                });
        });

    handle_shortcuts(ctx, app);
}

fn show_input_row(ui: &mut egui::Ui, app: &mut RTasksApp) {
    Frame::none()
        .fill(SELECTED_BACKGROUND)
        .inner_margin(Margin::symmetric(8.0, 7.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let input_width = (ui.available_width() - 170.0).max(240.0);
                let response = ui.add_sized(
                    [input_width, 24.0],
                    egui::TextEdit::singleline(&mut app.quick_input)
                        .hint_text("comprar pan mañana")
                        .font(TextStyle::Monospace)
                        .text_color(SELECTED_FOREGROUND)
                        .frame(false),
                );
                response.request_focus();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(summary_text(app));
                });
            });
        });
}

fn show_hint_row(ui: &mut egui::Ui, app: &RTasksApp) {
    Frame::none()
        .fill(BACKGROUND)
        .inner_margin(Margin::symmetric(8.0, 7.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Alt+1/2/3 estado · Alt+Q/W/E prioridad · Enter guardar · Shift+Enter seguir · Esc cerrar")
                        .font(mono_font(HINT_SIZE))
                        .color(MUTED),
                );

                if let Some(error) = &app.last_error {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(error).font(mono_font(HINT_SIZE)).color(Color32::RED));
                    });
                }
            });
        });
}

fn apply_command_palette_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = BACKGROUND;
    style.visuals.window_fill = BACKGROUND;
    style.visuals.extreme_bg_color = BACKGROUND;
    style.visuals.widgets.inactive.bg_fill = BACKGROUND;
    style.visuals.widgets.inactive.fg_stroke.color = FOREGROUND;
    style.visuals.widgets.active.fg_stroke.color = SELECTED_FOREGROUND;
    style.visuals.widgets.hovered.fg_stroke.color = SELECTED_FOREGROUND;
    style
        .text_styles
        .insert(TextStyle::Body, mono_font(TEXT_SIZE));
    style
        .text_styles
        .insert(TextStyle::Monospace, mono_font(TEXT_SIZE));
    ctx.set_style(style);
}

fn mono_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

fn handle_shortcuts(ctx: &egui::Context, app: &mut RTasksApp) {
    let mut save = false;
    let mut save_and_continue = false;
    let mut close = false;
    let mut consumed_suffix: Option<char> = None;

    ctx.input(|input| {
        if input.key_pressed(Key::Enter) {
            if input.modifiers.shift {
                save_and_continue = true;
            } else {
                save = true;
            }
        }

        if input.key_pressed(Key::Escape) {
            close = true;
        }

        if input.modifiers.alt {
            if input.key_pressed(Key::Num1) {
                toggle_selected_status(app, TaskStatus::Todo);
                consumed_suffix = Some('1');
            }
            if input.key_pressed(Key::Num2) {
                toggle_selected_status(app, TaskStatus::Doing);
                consumed_suffix = Some('2');
            }
            if input.key_pressed(Key::Num3) {
                toggle_selected_status(app, TaskStatus::Done);
                consumed_suffix = Some('3');
            }
            if input.key_pressed(Key::Q) {
                toggle_selected_priority(app, Priority::High);
                consumed_suffix = Some('q');
            }
            if input.key_pressed(Key::W) {
                toggle_selected_priority(app, Priority::Medium);
                consumed_suffix = Some('w');
            }
            if input.key_pressed(Key::E) {
                toggle_selected_priority(app, Priority::Low);
                consumed_suffix = Some('e');
            }
        }
    });

    if let Some(suffix) = consumed_suffix {
        strip_shortcut_suffix(&mut app.quick_input, suffix);
    }

    if save_and_continue {
        app.save_quick_input_and_continue();
    }

    if save {
        app.save_quick_input();
    }

    if close {
        app.close_quick_add();
    }
}

fn strip_shortcut_suffix(input: &mut String, _suffix: char) {
    while input
        .chars()
        .last()
        .is_some_and(|last| matches!(last.to_ascii_lowercase(), '1' | '2' | '3' | 'q' | 'w' | 'e'))
    {
        input.pop();
    }
}

fn toggle_selected_status(app: &mut RTasksApp, status: TaskStatus) {
    app.selected_status = if app.selected_status.as_ref() == Some(&status) {
        None
    } else {
        Some(status)
    };
}

fn toggle_selected_priority(app: &mut RTasksApp, priority: Priority) {
    app.selected_priority = if app.selected_priority.as_ref() == Some(&priority) {
        None
    } else {
        Some(priority)
    };
}

fn summary_text(app: &RTasksApp) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    append_summary_part(
        &mut job,
        status_label(app.selected_status.as_ref()),
        app.selected_status.is_some(),
    );
    append_summary_part(&mut job, "   ", true);
    append_summary_part(
        &mut job,
        format!("prio:{}", priority_label(app.selected_priority.as_ref())),
        app.selected_priority.is_some(),
    );
    job
}

fn append_summary_part(job: &mut egui::text::LayoutJob, text: impl AsRef<str>, active: bool) {
    job.append(
        text.as_ref(),
        0.0,
        egui::TextFormat {
            font_id: mono_font(TEXT_SIZE),
            color: if active { SELECTED_FOREGROUND } else { MUTED },
            ..Default::default()
        },
    );
}

fn status_label(status: Option<&TaskStatus>) -> &'static str {
    match status {
        Some(TaskStatus::Doing) => "DOING",
        Some(TaskStatus::Done) => "DONE",
        None | Some(TaskStatus::Todo) => "TODO",
    }
}

fn priority_label(priority: Option<&Priority>) -> &'static str {
    match priority {
        Some(Priority::High) => "alta",
        Some(Priority::Low) => "baja",
        None | Some(Priority::Medium) => "media",
    }
}

pub fn open(app: &mut RTasksApp) {
    app.mode = AppMode::QuickAdd;
}
