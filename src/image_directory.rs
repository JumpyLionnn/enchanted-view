use std::{path::{PathBuf, Path}, fs, sync::mpsc::{self, Receiver}, slice::Iter};
use image::ImageFormat;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, event::{CreateKind, Event, EventKind, RemoveKind, ModifyKind, RenameMode}};



pub struct ImageDirectory {
    name: String, // for cache
    index: usize,
    children: Vec<PathBuf>,
    // for file system changes
    receiver: Receiver<Change>,
    _watcher: RecommendedWatcher
}

impl ImageDirectory {
    pub fn new(image_path: &PathBuf) -> Result<Self, notify::Error> {
        let parent = image_path.parent().expect("There isn't a parent path for the image path");
        let image_entries = find_image_files(parent);
        let current_index = image_entries.iter().position(|path| path == image_path).expect("Couldn't find the current image.");
        let name = image_path.file_name().expect("Couldn't extract the file name").to_string_lossy().to_string();
        let (mut watcher, receiver) = setup_watcher()?;
        watcher.watch(parent, RecursiveMode::NonRecursive)?;
        Ok(Self {
            name,
            index: current_index,
            children: image_entries,
            receiver: receiver,
            _watcher: watcher
        })
    }

    pub fn image_name(&self) -> &String {
        &self.name
    }

    pub fn image_index(&self) -> usize {
        self.index
    }

    pub fn current_image_path(&self) -> &PathBuf {
        &self.children[self.index]
    }

    pub fn current_directory_path(&self) -> PathBuf {
        self.children[self.index].parent().expect("The parent path should exist.").to_path_buf()
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

    /// returns if an image reload is required
    pub fn check_for_changes(&mut self) -> bool {
        if let Ok(change) = self.receiver.try_recv() {
            match change {
                Change::Create(path) => {
                    let index = self.children.iter().position(|child_path| *child_path == path);
                    if let Some(index) = index {
                        eprintln!("File was created that already existed");
                        index == self.index
                    }
                    else {
                        // TODO(maybe): sort the children and then insert it at the right place
                        self.children.push(path);
                        false
                    }
                },
                Change::Remove(path) => {
                    let index = self.children.iter().position(|child_path| *child_path == path);
                    if let Some(index) = index {
                        self.children.remove(index);
                        let reload = index == self.index;
                        if index <= self.index {
                            self.index -= 1;
                        }
                        reload
                    }
                    else {
                        eprintln!("Got remove event, file didnt exist.");
                        false 
                    }
                },
                Change::FileChange(path) => {
                    self.children
                        .iter()
                        .position(|child_path| *child_path == path)
                        .is_some_and(|index| index == self.index)
                },
                Change::Rename(from, to) => {
                    let index = self.children.iter().position(|child_path| *child_path == from);
                    if let Some(index) = index {
                        if index == self.index {
                            self.name = to.file_name().expect("Couldn't extract the file name").to_string_lossy().to_string();
                        }
                        self.children[index] = to;
                    }
                    else {
                        eprintln!("Got rename event, file didnt exist.");
                    }
                    false
                },
            }
        }
        else { false }
    }
}


fn find_image_files(path: &Path) -> Vec<PathBuf> {
    fs::read_dir(path).expect("Couldn't read the parent dir")
        .filter_map(|element| {
            element.ok().and_then(|entry| {
                if is_image_file(&entry.path()) {
                    Some(entry.path())
                }
                else {
                    None
                }
            })
        })
        .collect::<Vec<PathBuf>>()
}

pub trait ImageFormatEx {
    fn iterator() -> Iter<'static, image::ImageFormat>;
}

impl ImageFormatEx for ImageFormat {
    fn iterator() -> Iter<'static, ImageFormat> {
        static FORMATS: [ImageFormat; 15] = [
            ImageFormat::Avif, 
            ImageFormat::Bmp, 
            ImageFormat::Dds, 
            ImageFormat::Farbfeld,
            ImageFormat::Gif,
            ImageFormat::Hdr,
            ImageFormat::Ico,
            ImageFormat::Jpeg,
            ImageFormat::OpenExr,
            ImageFormat::Png,
            ImageFormat::Pnm,
            ImageFormat::Qoi,
            ImageFormat::Tga,
            ImageFormat::Tiff,
            ImageFormat::WebP
        ];
        FORMATS.iter()
    }
}

enum Change {
    Create(PathBuf),
    Remove(PathBuf),
    FileChange(PathBuf),
    Rename(PathBuf, PathBuf)
}

fn setup_watcher() -> notify::Result<(RecommendedWatcher, Receiver<Change>)> {
    let (sender, receiver) = mpsc::channel();
    let mut last_rename = None;
    Ok((notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(mut event) = res {
            match event.kind {
                EventKind::Any => {}
                EventKind::Access(_) => {},
                EventKind::Create(kind) => {
                    match kind {
                        CreateKind::Folder => {}
                        _ => {
                            for path in event.paths {
                                if is_image_file(&path) {
                                    let _  = sender.send(Change::Create(path));
                                }
                            }
                        }
                    }
                },
                EventKind::Modify(kind) =>  {
                    match kind {
                        ModifyKind::Name(mode) => {
                            match mode {
                                RenameMode::From => {
                                    assert_eq!(event.paths.len(), 1);
                                    assert!(last_rename.is_none());
                                    last_rename = Some(event.paths.pop().expect("Couldn't take to value"));
                                },
                                RenameMode::To => {
                                    assert_eq!(event.paths.len(), 1);
                                    if let Some(from) = last_rename.take() {
                                        if is_image_file(&event.paths[0]) {
                                            let _  = sender.send(Change::Rename(from, event.paths.pop().expect("Couldn't take to value")));
                                        }
                                    }
                                    else {
                                        eprintln!("Unable to rename, got To before From.");
                                    }
                                }
                                RenameMode::Both => {
                                    assert_eq!(event.paths.len(), 2);
                                    let to = event.paths.pop().expect("Couldn't take to value");
                                    if is_image_file(&to) {
                                        let from = event.paths.pop().expect("Couldn't take to value");
                                        let _  = sender.send(Change::Rename(from, to));
                                    }
                                }
                                _other => {
                                    eprintln!("Unknown rename event.");
                                }
                            }
                        }
                        _other => {
                            for path in event.paths {
                                if is_image_file(&path) {
                                    let _  = sender.send(Change::FileChange(path));
                                }
                            }
                        }
                    }
                },
                EventKind::Remove(kind) =>  {
                    match kind {
                        RemoveKind::Folder => {}
                        _ => {
                            for path in event.paths {
                                if is_image_file(&path) {
                                    let _  = sender.send(Change::Remove(path));
                                }
                            }
                        }
                    }
                },
                EventKind::Other =>  {},
            }
        }
    })?, receiver))
}

pub fn is_image_file(path: &PathBuf) -> bool {
    image::ImageFormat::from_path(path).is_ok_and(|format| format.can_read())
}