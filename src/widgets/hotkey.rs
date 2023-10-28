use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
#[serde(remote = "egui::KeyboardShortcut")]
pub struct KeyboardShortcutRef {
    pub modifiers: egui::Modifiers,
    pub key: egui::Key,
}

pub struct Hotkey<'a> {
    binding: &'a mut egui::KeyboardShortcut,
    id: egui::Id
}

impl<'a> Hotkey<'a> {
    pub fn new(key: &'a mut egui::KeyboardShortcut, id: impl std::hash::Hash) -> Self {
        Self {
            binding: key,
            id: egui::Id::new(id)
        }
    }
}

impl<'a> egui::Widget for Hotkey<'a>
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let size = ui.spacing().interact_size;
        let (rect, mut response) = ui.allocate_exact_size(size * egui::vec2(2.0, 1.2), egui::Sense::click());

        let mut expecting = ui.data(|reader| reader.get_temp(self.id)).unwrap_or(false);

        if response.clicked() {
            expecting = !expecting;
        }

        if expecting {
            if response.clicked_elsewhere() {
                expecting = false;
            } else if ui.input(|reader| reader.key_pressed(egui::Key::Enter)) {
                expecting = false;
            } else {
                let keyboard = ui.input(|reader| {
                    reader.events.iter().find_map(|e| match e {
                        egui::Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            repeat: _
                        } => Some(egui::KeyboardShortcut::new(*modifiers, *key)),
                        _ => None,
                    })
                });

                if let Some(shortcut) = keyboard {
                    *self.binding = shortcut;
                    response.mark_changed();
                    expecting = false;
                }
            }
        }

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact_selectable(&response, expecting);
            ui.painter()
                .rect_filled(rect, egui::Rounding::same(2.0), visuals.bg_fill);

            let mut text = ui.ctx().format_shortcut(&self.binding);
            if expecting {
                text = format!("> {text} <");
            }

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::default(),
                visuals.text_color(),
            );
        }

        ui.data_mut(|writer| writer.insert_temp(self.id, expecting));
        response
    }
}

pub fn key_bind_widget(ui: &mut egui::Ui, name: &str, bind: &mut egui::KeyboardShortcut, default: egui::KeyboardShortcut) {
    ui.label(name);
    ui.add(Hotkey::new(bind, name));
    if ui.add_enabled(*bind != default, egui::Button::new("Reset").min_size(egui::vec2(1.0, ui.spacing().interact_size.y * 1.2))).clicked() {
        *bind = default;
    }
    ui.end_row();
}