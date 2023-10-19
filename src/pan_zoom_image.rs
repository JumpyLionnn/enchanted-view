use egui::{Sense, Color32};
use crate::egui_extensions::{PainterEx, Vec2Ex};

pub struct PanZoomImage {
    pub constrain_to_image: bool,
    pub always_center: bool,
    texture_handle: egui::TextureHandle,
    // The actual texture dimensions
    texture_size: egui::Vec2,
    // image size will switch the axises on rotation
    image_size: egui::Vec2,
    offset: egui::Vec2,
    pub scale: f32,
    checkers_mesh: egui::Shape,
    last_rect: egui::Rect,
    last_image_rect: egui::Rect,
    min_scale: f32,
    pub max_scale: f32,
    fit_request: bool
}

impl PanZoomImage {
    pub fn new(constrain_to_image: bool, always_center: bool, texture_handle: egui::TextureHandle, texture_size: egui::Vec2) -> Self {
        Self {
            constrain_to_image,
            always_center,
            texture_handle,
            offset: egui::vec2(0.0, 0.0),
            scale: 1.0,
            checkers_mesh: egui::Shape::Noop,
            last_rect: egui::Rect { min: egui::pos2(0.0, 0.0), max: egui::pos2(0.0, 0.0) },
            last_image_rect: egui::Rect { min: egui::pos2(0.0, 0.0), max: egui::pos2(0.0, 0.0) },
            min_scale: 0.0,
            max_scale: 32.0,
            texture_size: texture_size,
            image_size: texture_size,
            fit_request: true
        }
    }

    pub fn zoom_to_original(&mut self) {
        self.set_zoom(1.0, self.last_image_rect.center().to_vec2());
    }

    pub fn zoom_to_fit(&mut self) {
        let new_scale = self.calc_fit_scale(self.last_rect);
        self.set_zoom(new_scale, self.last_image_rect.center().to_vec2());
    }

    pub fn zoom_in(&mut self) {
        // TODO:  animation using tweened
        // zoom towards the center
        self.set_zoom(self.scale + self.scale * 0.1, self.last_image_rect.center().to_vec2());
    }

    pub fn zoom_out(&mut self) {
        // TODO:  animation using tweened
        // zoom towards the center
        self.set_zoom(self.scale - self.scale * 0.1, self.last_image_rect.center().to_vec2());
    }

    pub fn can_zoom_in(&self) -> bool {
        self.scale < self.max_scale
    }

    pub fn can_zoom_out(&self) -> bool {
        self.scale > self.min_scale
    }

    fn world_to_screen(&self, world: egui::Vec2) -> egui::Vec2 {
        egui::Vec2 {
            x: (world.x - self.offset.x) * self.scale, 
            y: (world.y - self.offset.y) * self.scale
        }
    }

    fn screen_to_world(&self, screen: egui::Vec2)  -> egui::Vec2{
        egui::Vec2 {
            x: screen.x / self.scale + self.offset.x, 
            y: screen.y / self.scale + self.offset.y
        }
    }

    fn calc_fit_scale(&mut self, rect: egui::Rect) -> f32 {
        f32::min(rect.width() / self.image_size.x, rect.height() / self.image_size.y)
    }

    fn set_zoom(&mut self, zoom_value: f32, anchor: egui::Vec2) {
        let before_zoom = self.screen_to_world(anchor);
        self.scale = zoom_value;
        self.scale = self.scale.clamp(self.min_scale, self.max_scale);
        let after_zoom = self.screen_to_world(anchor);
        self.offset += before_zoom - after_zoom;
    }

    pub fn update(&mut self, ui: &mut egui::Ui, flip_horizontal: bool, flip_vertical: bool, rotation: usize) {
        const DEBUG: bool = false;
        // TODO: animate the scaling to be smooth

        if rotation % 2 == 0 {
            self.image_size = self.texture_size;
        }
        else {
            self.image_size = egui::vec2(self.texture_size.y, self.texture_size.x);
        }

        let mouse_pos = ui.input(|input| input.pointer.latest_pos().unwrap_or(egui::pos2(0.0, 0.0))).to_vec2();

        let (rect, res) = ui.allocate_at_least(ui.available_size(), Sense::drag());
        self.last_rect = rect;
        // them min scale that the image can fit on the screen
        self.min_scale = self.calc_fit_scale(rect).min(1.0);
        if self.fit_request {
            // when an image changes or on the first frame we change the scale of the image to fit
            self.zoom_to_fit();
            self.fit_request = false;
        }
        // panning
        if res.dragged() {
            self.offset -=  res.drag_delta() / self.scale;
        }
        // zooming
        let scroll_delta = ui.input(|input| input.scroll_delta);
        if scroll_delta.y > 0.0 {
            self.set_zoom(self.scale + self.scale * 0.1, mouse_pos);
        }
        if scroll_delta.y < 0.0 {
            self.set_zoom(self.scale - self.scale * 0.1, mouse_pos);
        }
        self.scale = self.scale.clamp(self.min_scale, self.max_scale);

        // calculating min and max offset to constrain the pan to the image boundaries
        if self.constrain_to_image {
            let min_offset = rect.min.to_vec2() - rect.min.to_vec2() / self.scale;
            let max_offset = ((self.image_size * self.scale - rect.size()) / self.scale + min_offset).max(min_offset);
            self.offset = self.offset.clamp(min_offset, max_offset);
            if DEBUG {
                ui.painter().debug_label(rect.min + egui::vec2(0.0, 20.0), format!("offset: {:?}, min: {:?}", self.offset, min_offset));
                ui.painter().debug_label(rect.min + egui::vec2(0.0, 40.0), format!("offset: {:?}, max: {:?}", self.offset, max_offset));
            }
        }

        // centering
        if self.always_center {
            let free_space = rect.size() - self.image_size * self.scale;
            if free_space.x > 0.0 {
                self.offset.x = -free_space.x / 2.0 / self.scale;
                // the scaling changes the image offset too so its needed to be corrected
                self.offset.x +=  (rect.min.x * self.scale - rect.min.x) / self.scale;
            }
            if free_space.y > 0.0 {
                self.offset.y = -free_space.y / 2.0 / self.scale;
                // the scaling changes the image offset too so its needed to be corrected
                self.offset.y +=  (rect.min.y * self.scale - rect.min.y) / self.scale;
                
            }
            if DEBUG {
                let x_middle = rect.min.x + rect.width() / 2.0;
                ui.painter().debug_line([egui::pos2(x_middle, rect.min.y), egui::pos2(x_middle, rect.min.y + free_space.y / 2.0)]);
                ui.painter().debug_line([egui::pos2(x_middle, rect.max.y), egui::pos2(x_middle, rect.max.y - free_space.y / 2.0)]);
            }
        }

        
        let image_min = self.world_to_screen(rect.min.to_vec2());
        let image_max = self.world_to_screen(rect.min.to_vec2() + self.image_size);
        let image_rect = egui::Rect {
            min: image_min.to_pos2().clamp(rect.min, rect.max),
            max: image_max.to_pos2().clamp(rect.min, rect.max)
        };

        if self.last_image_rect != image_rect {
            self.regenerate_checkerboard(image_rect);
            self.last_image_rect = image_rect;
        }

        // TODO: find a way to remove the clone
        // A better way to do the checkers background is using texture tiling(in the shader) but this is not available in egui at the moment
        // Waiting for issue #3481 in egui
        let mesh = self.checkers_mesh.clone();
        ui.painter().add(mesh);

        let mesh = self.generate_image_mesh(image_min, image_max, image_rect, rect, flip_horizontal, flip_vertical, rotation);
        ui.painter().add(egui::Shape::mesh(mesh));

        if DEBUG {
            ui.painter().debug_stroke(rect);
            ui.painter().debug_stroke(image_rect);
            ui.painter().debug_label(rect.min, format!("scale: {}, min: {}, max: {}", self.scale, self.min_scale, self.max_scale));
        }
    }

    fn generate_image_mesh(&self, image_min: egui::Vec2, image_max: egui::Vec2, image_rect: egui::Rect, rect: egui::Rect, flip_horizontal: bool, flip_vertical: bool, rotation: usize) -> egui::Mesh {
        use egui::epaint::Vertex;

        let mut clipped = self.clipping_uv(image_min, image_max, rect);
        // apply flipping
        let max = clipped.max;
        if flip_horizontal {
            clipped.max.x = clipped.min.x;
            clipped.min.x = max.x;
        }
        if flip_vertical {
            clipped.max.y = clipped.min.y;
            clipped.min.y = max.y;
        }

        // rotating
        let uv_center = egui::vec2(0.5, 0.5);
        let left_top = clipped.left_top().rotate90_around(uv_center, rotation);
        let right_top = clipped.right_top().rotate90_around(uv_center, rotation);
        let right_bottom = clipped.right_bottom().rotate90_around(uv_center, rotation);
        let left_bottom = clipped.left_bottom().rotate90_around(uv_center, rotation);

        let mut mesh = egui::Mesh::with_texture(self.texture_handle.id());
        let idx = mesh.vertices.len() as u32;
        mesh.reserve_triangles(2);
        mesh.reserve_vertices(4);

        mesh.add_triangle(idx + 0, idx + 1, idx + 2);
        mesh.add_triangle(idx + 2, idx + 3, idx + 0);

        mesh.vertices.push(Vertex {
            pos: image_rect.left_top(),
            uv: left_top,
            color: Color32::WHITE,
        });
        mesh.vertices.push(Vertex {
            pos: image_rect.right_top(),
            uv: right_top,
            color: Color32::WHITE,
        });
        mesh.vertices.push(Vertex {
            pos: image_rect.right_bottom(),
            uv: right_bottom,
            color: Color32::WHITE,
        });
        mesh.vertices.push(Vertex {
            pos: image_rect.left_bottom(),
            uv: left_bottom,
            color: Color32::WHITE,
        });

        mesh
    }

    

    fn clipping_uv(&self, image_min: egui::Vec2, image_max: egui::Vec2, rect: egui::Rect) -> egui::Rect {
        let mut uv_min_x = 0.0;
        if image_min.x < rect.min.x {
            uv_min_x = ((rect.min.x - image_min.x) / self.scale) / self.image_size.x;
        }
        let mut uv_min_y = 0.0;
        if image_min.y < rect.min.y {
            uv_min_y = ((rect.min.y - image_min.y) / self.scale) / self.image_size.y;
        }
        let mut uv_max_x = 1.0;
        if image_max.x > rect.max.x {
            uv_max_x = (self.image_size.x + (rect.max.x - image_max.x) / self.scale) / self.image_size.x;
        }
        let mut uv_max_y = 1.0;
        if image_max.y > rect.max.y {
            uv_max_y = (self.image_size.y + (rect.max.y - image_max.y) / self.scale) / self.image_size.y;
        }


        egui::Rect {
            min: (egui::vec2(uv_min_x, uv_min_y)).to_pos2(),
            max: (egui::vec2(uv_max_x, uv_max_y)).to_pos2(),
        }
    }

    fn regenerate_checkerboard(&mut self, area: egui::Rect) {
        const RECT_SIZE: f32 = 8.0;
        let size = area.size();
        let mut mesh = egui::Mesh::default();
        let checker_count = (size / RECT_SIZE).ceil();
        for row in 0..checker_count.y as u32 {
            for column in 0..checker_count.x as u32 {
                let x = column as f32 * RECT_SIZE + area.min.x;
                let y = row as f32 * RECT_SIZE + area.min.y;
                let rect = egui::Rect {
                    min: egui::pos2(x, y),
                    max: egui::pos2(x + RECT_SIZE, y + RECT_SIZE).min(area.max)
                };
                let color = if (row + column) % 2 == 0 {Color32::WHITE} else {Color32::LIGHT_GRAY};
                mesh.add_colored_rect(rect, color);
            }
        }
        self.checkers_mesh = egui::Shape::mesh(mesh);
    }
}

