use egui::Sense;
use crate::egui_extensions::PainterEx;

pub struct PanZoomImage {
    pub constrain_to_image: bool,
    pub always_center: bool,
    pub texture_handle: egui::TextureHandle,
    offset: egui::Vec2,
    scale: f32,
}

impl PanZoomImage {
    pub fn new(constrain_to_image: bool, always_center: bool, texture_handle: egui::TextureHandle) -> Self {
        Self {
            constrain_to_image,
            always_center,
            texture_handle,
            offset: egui::vec2(0.0, 0.0),
            scale: 1.0
        }
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

    pub fn update(&mut self, ui: &mut egui::Ui, image_size: egui::Vec2) {
        const DEBUG: bool = false;

        let mouse_pos = ui.input(|input| input.pointer.latest_pos().unwrap_or(egui::pos2(0.0, 0.0))).to_vec2();

        let (rect, res) = ui.allocate_at_least(ui.available_size(), Sense::drag());
        // them min scale that the image can fit on the screen
        let min_scale = f32::min(rect.width() / image_size.x, rect.height() / image_size.y).min(1.0);
        let max_scale = 32.0;
        if ui.ctx().frame_nr() == 1 {
            // on first frame we set the min scale
            self.scale = min_scale;
        }
        // input
        if res.dragged() {
            self.offset -=  res.drag_delta() / self.scale;
        }
        // zooming
        let scroll_delta = ui.input(|input| input.scroll_delta);
        let before_zoom = self.screen_to_world(mouse_pos);
        if scroll_delta.y > 0.0 {
            self.scale += self.scale * 0.1;
        }
        if scroll_delta.y < 0.0 {
            self.scale -= self.scale * 0.1;
        }
        self.scale = self.scale.clamp(min_scale, max_scale);
        let after_zoom = self.screen_to_world(mouse_pos);
        self.offset += before_zoom - after_zoom;

        // calculating min and max offset to constrain the pan to the image boundaries
        if self.constrain_to_image {
            let min_offset = rect.min.to_vec2() - rect.min.to_vec2() / self.scale;
            let max_offset = ((image_size * self.scale - rect.size()) / self.scale + min_offset).max(min_offset);
            self.offset = self.offset.clamp(min_offset, max_offset);
            if DEBUG {
                ui.painter().debug_label(rect.min + egui::vec2(0.0, 20.0), format!("offset: {:?}, min: {:?}", self.offset, min_offset));
                ui.painter().debug_label(rect.min + egui::vec2(0.0, 40.0), format!("offset: {:?}, max: {:?}", self.offset, max_offset));
            }
        }

        // centering
        if self.always_center {
            let free_space = rect.size() - image_size * self.scale;
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
        let image_max = self.world_to_screen(rect.min.to_vec2() + image_size);
        let image_rect = egui::Rect {
            min: image_min.to_pos2().clamp(rect.min, rect.max),
            max: image_max.to_pos2().clamp(rect.min, rect.max)
        };

        if DEBUG {
            ui.painter().debug_stroke(rect);
            ui.painter().debug_stroke(image_rect);
        }

        // calculating the uv of the texture to clip the invisible parts
        let mut uv_min_x = 0.0;
        if image_min.x < rect.min.x {
            uv_min_x = ((rect.min.x - image_min.x) / self.scale) / image_size.x;
        }
        let mut uv_min_y = 0.0;
        if image_min.y < rect.min.y {
            uv_min_y = ((rect.min.x - image_min.y) / self.scale) / image_size.y;
        }
        let mut uv_max_x = 1.0;
        if image_max.x > rect.max.x {
            uv_max_x = (image_size.x + (rect.max.x - image_max.x) / self.scale) / image_size.x;
        }
        let mut uv_max_y = 1.0;
        if image_max.y > rect.max.y {
            uv_max_y = (image_size.y + (rect.max.y - image_max.y) / self.scale) / image_size.y;
        }
        let uv = egui::Rect {
            min: egui::pos2(uv_min_x, uv_min_y),
            max: egui::pos2(uv_max_x, uv_max_y),
        };
        // TODO: draw checkers background for images with transparency
        ui.painter().image(self.texture_handle.id(), image_rect, uv, egui::Color32::WHITE);

        
        if DEBUG {
            ui.painter().debug_label(rect.min, format!("scale: {}, min: {}, max: {}", self.scale, min_scale, max_scale));
        }
       

    }
}