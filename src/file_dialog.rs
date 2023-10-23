use std::path::PathBuf;

pub struct FileDialog {
    dialog: rfd::FileDialog
}

impl FileDialog {
    pub fn new(frame: &eframe::Frame) -> Self {
        Self {
            dialog: rfd::FileDialog::new().set_parent(frame)
        }
    }

    pub fn directory(mut self, directory: Option<PathBuf>) -> Self {
        if let Some(directory) = directory {
            self.dialog = self.dialog.set_directory(directory);
        } 
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.dialog = self.dialog.set_title(title);
        self
    }

    pub fn add_filter(mut self, name: impl Into<String>, extensions: &[impl ToString]) -> Self {
        self.dialog = self.dialog.add_filter(name, extensions);
        self
    }

    pub fn pick_file(self, context: &egui::Context) -> FileDialogHandle {
        let (sender, receiver) = std::sync::mpsc::channel();
        let context = context.clone();
        let thread_handle = std::thread::spawn(move || {
            let file = self.dialog.pick_file();
            if let Some(path) = file {
                let _ = sender.send(path);
            }
            context.request_repaint();
        });
        FileDialogHandle { _thread_handle: thread_handle, receiver }
    }
}

pub struct FileDialogHandle {
    _thread_handle: std::thread::JoinHandle<()>,
    receiver: std::sync::mpsc::Receiver<PathBuf>
}

impl FileDialogHandle {
    pub fn file_picked(&self) -> Option<PathBuf> {
        self.receiver.try_recv().ok()
    }
}