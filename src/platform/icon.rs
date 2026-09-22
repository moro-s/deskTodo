use eframe::egui;

const BG_TOP: [f32; 3] = [90.0, 165.0, 255.0];
const BG_BOTTOM: [f32; 3] = [45.0, 107.0, 242.0];
const RING: [f32; 3] = [216.0, 233.0, 255.0];
const CARD: [f32; 3] = [255.0, 255.0, 255.0];
const CHECK: [f32; 3] = [61.0, 127.0, 245.0];

fn coverage(sd: f32) -> f32 {
    (0.5 - sd).clamp(0.0, 1.0)
}

fn sd_rounded_rect(
    px: f32,
    py: f32,
    center_x: f32,
    center_y: f32,
    half_width: f32,
    half_height: f32,
    radius: f32,
) -> f32 {
    let qx = (px - center_x).abs() - (half_width - radius);
    let qy = (py - center_y).abs() - (half_height - radius);
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    let inside = qx.max(qy).min(0.0);
    outside + inside - radius
}

fn sd_segment(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let abx = bx - ax;
    let aby = by - ay;
    let apx = px - ax;
    let apy = py - ay;
    let t = ((apx * abx + apy * aby) / (abx * abx + aby * aby)).clamp(0.0, 1.0);
    let dx = apx - t * abx;
    let dy = apy - t * aby;
    (dx * dx + dy * dy).sqrt()
}

fn over(dst: &mut [f32; 4], src: [f32; 3], alpha: f32) {
    let alpha = alpha.clamp(0.0, 1.0);
    if alpha <= 0.0 {
        return;
    }
    let dst_alpha = dst[3];
    let out_alpha = alpha + dst_alpha * (1.0 - alpha);
    if out_alpha <= 0.0 {
        return;
    }
    for i in 0..3 {
        dst[i] = (src[i] * alpha + dst[i] * dst_alpha * (1.0 - alpha)) / out_alpha;
    }
    dst[3] = out_alpha;
}

pub(crate) fn calendar_icon_data() -> egui::IconData {
    let size = 64u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let mut pixel = [0.0f32; 4];

            let gradient = ((px + py) / 126.0).clamp(0.0, 1.0);
            let bg = [
                BG_TOP[0] + (BG_BOTTOM[0] - BG_TOP[0]) * gradient,
                BG_TOP[1] + (BG_BOTTOM[1] - BG_TOP[1]) * gradient,
                BG_TOP[2] + (BG_BOTTOM[2] - BG_TOP[2]) * gradient,
            ];
            over(
                &mut pixel,
                bg,
                coverage(sd_rounded_rect(px, py, 32.0, 32.0, 31.0, 31.0, 15.0)),
            );
            over(
                &mut pixel,
                RING,
                coverage(sd_rounded_rect(px, py, 22.0, 13.0, 3.5, 8.0, 3.5)),
            );
            over(
                &mut pixel,
                RING,
                coverage(sd_rounded_rect(px, py, 42.0, 13.0, 3.5, 8.0, 3.5)),
            );
            over(
                &mut pixel,
                CARD,
                coverage(sd_rounded_rect(px, py, 32.0, 36.0, 23.0, 23.0, 9.0)),
            );
            let check = sd_segment(px, py, 21.0, 38.0, 29.0, 46.0)
                .min(sd_segment(px, py, 29.0, 46.0, 44.0, 31.0))
                - 3.1;
            over(&mut pixel, CHECK, coverage(check));

            rgba.extend_from_slice(&[
                pixel[0].clamp(0.0, 255.0).round() as u8,
                pixel[1].clamp(0.0, 255.0).round() as u8,
                pixel[2].clamp(0.0, 255.0).round() as u8,
                (pixel[3] * 255.0).clamp(0.0, 255.0).round() as u8,
            ]);
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
