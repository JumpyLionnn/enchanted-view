

pub fn generate_checkerboard_pattern(area: egui::Rect, rect_size: f32, colors: [egui::Color32; 2]) -> egui::Shape {
    let size = area.size();
    let mut mesh = egui::Mesh::default();
    let checker_count = (size / rect_size).ceil();
    for row in 0..checker_count.y as u32 {
        for column in 0..checker_count.x as u32 {
            let x = column as f32 * rect_size + area.min.x;
            let y = row as f32 * rect_size + area.min.y;
            let rect = egui::Rect {
                min: egui::pos2(x, y),
                max: egui::pos2(x + rect_size, y + rect_size).min(area.max)
            };
            let color = colors[((row + column) % 2) as usize];
            mesh.add_colored_rect(rect, color);
        }
    }
    egui::Shape::mesh(mesh)
}