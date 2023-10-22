use std::{path::PathBuf, fs};



pub struct ImageDirectory {
    name: String, // for cache
    index: usize,
    children: Vec<PathBuf>
}

impl ImageDirectory {
    pub fn new(image_path: &PathBuf) -> Self {
        let parent = image_path.parent().expect("There isn't a parent path for the image path");
        let image_entries = fs::read_dir(parent).expect("Couldn't read the parent dir")
            .filter_map(|element| {
                element.ok().and_then(|entry| {
                    if image::ImageFormat::from_path(entry.path()).is_ok() {
                        Some(entry.path())
                    }
                    else {
                        None
                    }
                })
            })
            .collect::<Vec<PathBuf>>();
        let current_index = image_entries.iter().position(|path| path == image_path).expect("Couldn't find the current image.");
        let name = image_path.file_name().expect("Couldn't extract the file name").to_string_lossy().to_string();
        Self {
            name,
            index: current_index,
            children: image_entries
        }
    }

    pub fn image_name(&self) -> &String {
        &self.name
    }

    pub fn image_index(&self) -> usize {
        self.index
    }

    pub fn count(&self) -> usize {
        self.children.len()
    }

    pub fn next_image(&mut self) -> &PathBuf {
        if self.index == self.children.len() - 1 {
            self.index = 0;
        }
        else {
            self.index += 1;
        }
        let path = &self.children[self.index];
        self.name = path.file_name().expect("Couldn't extract the file name").to_string_lossy().to_string(); 
        path
    }

    pub fn previous_image(&mut self) -> &PathBuf {
        if self.index == 0 {
            self.index = self.children.len() - 1;
        }
        else {
            self.index -= 1;
        }
        let path = &self.children[self.index];
        self.name = path.file_name().expect("Couldn't extract the file name").to_string_lossy().to_string(); 
        path
    }
}
