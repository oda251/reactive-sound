use eframe::egui;

pub struct LaneCursor {
    pub position: f32, // 0.0..1.0 within the lane
    pub color: egui::Color32,
}

/// Draw a single lane (one measure).
/// `events`: positions within the lane (0.0..1.0)
/// `note_color`: color for note markers
/// `cursor`: optional cursor line
pub fn draw_lane(
    ui: &mut egui::Ui,
    lane_width: f32,
    lane_height: f32,
    events: &[f32],
    note_color: egui::Color32,
    cursor: Option<&LaneCursor>,
    highlight: bool,
) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(lane_width, lane_height),
        egui::Sense::hover(),
    );

    let bg = if highlight {
        egui::Color32::from_gray(32)
    } else {
        egui::Color32::from_gray(25)
    };
    ui.painter().rect_filled(rect, 2.0, bg);

    // Beat lines (4 beats per measure)
    for beat in 1..4 {
        let x = rect.min.x + lane_width * (beat as f32 / 4.0);
        ui.painter().line_segment(
            [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
            egui::Stroke::new(1.0, egui::Color32::from_gray(45)),
        );
    }

    // Note markers
    for &pos in events {
        let x = rect.min.x + lane_width * pos;
        let note_rect = egui::Rect::from_min_size(
            egui::pos2(x - 1.5, rect.min.y + 2.0),
            egui::vec2(3.0, lane_height - 4.0),
        );
        ui.painter().rect_filled(note_rect, 1.0, note_color);
    }

    // Cursor line
    if let Some(c) = cursor {
        let cx = rect.min.x + lane_width * c.position;
        ui.painter().line_segment(
            [egui::pos2(cx, rect.min.y), egui::pos2(cx, rect.max.y)],
            egui::Stroke::new(2.0, c.color),
        );
    }
}

/// Split positions (0.0..1.0 in full loop) into per-measure buckets.
pub fn split_into_measures(positions: &[f32], measures: usize) -> Vec<Vec<f32>> {
    let mut result = vec![Vec::new(); measures];
    for &pos in positions {
        let m = ((pos * measures as f32) as usize).min(measures - 1);
        let pos_in_m = (pos * measures as f32).fract();
        result[m].push(pos_in_m);
    }
    result
}
