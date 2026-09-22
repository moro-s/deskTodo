use eframe::egui;

pub(crate) fn calendar_icon_data() -> egui::IconData {
    let size = 64u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let ix = x as i32;
            let iy = y as i32;
            let mut color = [45, 92, 180, 255];
            if (8..14).contains(&iy) && (12..52).contains(&ix) {
                color = [235, 105, 95, 255];
            }
            if (3..9).contains(&iy) && ((18..24).contains(&ix) || (40..46).contains(&ix)) {
                color = [235, 105, 95, 255];
            }
            for grid_y in 0..2 {
                for grid_x in 0..3 {
                    let cell_x = 12 + grid_x * 14;
                    let cell_y = 22 + grid_y * 14;
                    if (cell_x..cell_x + 8).contains(&ix) && (cell_y..cell_y + 8).contains(&iy) {
                        color = [255, 255, 255, 255];
                    }
                }
            }
            rgba.extend_from_slice(&color);
        }
    }
    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}

pub(crate) fn tray_icon_from(icon_data: &egui::IconData) -> Option<tray_icon::Icon> {
    tray_icon::Icon::from_rgba(
        icon_data.rgba.clone(),
        icon_data.width,
        icon_data.height,
    )
    .ok()
}
