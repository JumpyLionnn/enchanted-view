use std::path::PathBuf;

use crate::{file_dialog::{FileDialogHandle, FileDialog}, utilities::format_path};



pub struct PathPicker<'a> {
    path: &'a mut PathBuf,
    frame: &'a eframe::Frame,
    state: &'a mut PathPickerState
}

impl<'a> PathPicker<'a> {
    pub fn new(path: &'a mut PathBuf, frame: &'a eframe::Frame, state: &'a mut PathPickerState) -> Self {
        Self { path, frame, state }
    }
}

impl<'a> egui::Widget for PathPicker<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut changed = false;
        let mut res = ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            let button = ui.button("Browse");
            if button.clicked() {
                self.state.file_dialog_handle = Some(FileDialog::new(self.frame).title("Choose a folder").directory(Some(self.path.clone())).pick_directory(ui.ctx()))
            }
            if let Some(handle) = self.state.file_dialog_handle.as_ref() {
                if let Some(path) = handle.file_picked() {
                    *self.path = path;
                    self.state.current_path = None;
                }
            }

            let path = self.state.current_path.as_ref().and_then(|path| Some(path.to_owned()))
                .or_else(|| self.path.to_str().and_then(|p| Some(format_path(p))));
            let enabled = path.is_some();
            let mut text = path.unwrap_or_else(|| self.path.to_string_lossy().to_string());
            let res = ui.add_enabled(enabled, egui::TextEdit::singleline(&mut text).hint_text("Path").desired_width(ui.available_width()));
            if res.lost_focus() {
                *self.path = PathBuf::from(&text);
                changed = true;
            }
            
            if res.changed() {
                self.state.current_path = Some(text);
            }
        }).response;
        if changed {
            res.mark_changed();
        }
        res
    }
}

#[derive(Default)]
pub struct PathPickerState {
    current_path: Option<String>,
    file_dialog_handle: Option<FileDialogHandle>
}