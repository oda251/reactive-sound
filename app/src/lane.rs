use eframe::egui;

/// Describes how to render a cursor line.
pub struct CursorStyle {
    pub color: egui::Color32,
    pub width: f32,
}

/// Describes how to render a note marker.
pub struct NoteStyle {
    pub width: f32,
    pub rounding: f32,
}

impl Default for NoteStyle {
    fn default() -> Self {
        Self {
            width: 3.0,
            rounding: 1.0,
        }
    }
}

/// Configuration for draw_lane. All visual behavior is injected via closures
/// and style structs, so the same component can serve score, input, or custom lanes.
pub struct LaneConfig<'a> {
    pub width: f32,
    pub height: f32,
    /// Background color. Receives `(rect, painter)`.
    pub bg_color: egui::Color32,
    /// Grid line positions (0.0..1.0). Default: quarter-note beats at 0.25, 0.5, 0.75.
    pub grid_lines: &'a [f32],
    pub grid_color: egui::Color32,
    /// Note style.
    pub note_style: NoteStyle,
    /// Called per note to determine its color. Receives note position (0.0..1.0).
    pub note_color: &'a dyn Fn(f32) -> egui::Color32,
    /// Cursor position (0.0..1.0) and style. None = no cursor.
    pub cursor: Option<(f32, CursorStyle)>,
}

const DEFAULT_GRID: &[f32] = &[0.25, 0.5, 0.75];

impl<'a> LaneConfig<'a> {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            bg_color: egui::Color32::from_gray(25),
            grid_lines: DEFAULT_GRID,
            grid_color: egui::Color32::from_gray(45),
            note_style: NoteStyle::default(),
            note_color: &|_| egui::Color32::from_rgb(60, 220, 60),
            cursor: None,
        }
    }

    pub fn bg(mut self, color: egui::Color32) -> Self {
        self.bg_color = color;
        self
    }

    #[allow(dead_code)]
    pub fn grid(mut self, lines: &'a [f32], color: egui::Color32) -> Self {
        self.grid_lines = lines;
        self.grid_color = color;
        self
    }

    pub fn notes(mut self, color_fn: &'a dyn Fn(f32) -> egui::Color32) -> Self {
        self.note_color = color_fn;
        self
    }

    #[allow(dead_code)]
    pub fn note_style(mut self, style: NoteStyle) -> Self {
        self.note_style = style;
        self
    }

    pub fn cursor(mut self, position: f32, style: CursorStyle) -> Self {
        self.cursor = Some((position, style));
        self
    }
}

/// Draw a single lane with the given configuration and note events.
pub fn draw_lane(ui: &mut egui::Ui, config: &LaneConfig, events: &[f32]) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(config.width, config.height),
        egui::Sense::hover(),
    );

    // Background
    ui.painter().rect_filled(rect, 2.0, config.bg_color);

    // Grid lines
    for &pos in config.grid_lines {
        let x = rect.min.x + config.width * pos;
        ui.painter().line_segment(
            [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
            egui::Stroke::new(1.0, config.grid_color),
        );
    }

    // Notes
    let half_w = config.note_style.width / 2.0;
    let pad = 2.0;
    for &pos in events {
        let x = rect.min.x + config.width * pos;
        let note_rect = egui::Rect::from_min_size(
            egui::pos2(x - half_w, rect.min.y + pad),
            egui::vec2(config.note_style.width, config.height - pad * 2.0),
        );
        let color = (config.note_color)(pos);
        ui.painter()
            .rect_filled(note_rect, config.note_style.rounding, color);
    }

    // Cursor
    if let Some((pos, ref style)) = config.cursor {
        let cx = rect.min.x + config.width * pos;
        ui.painter().line_segment(
            [egui::pos2(cx, rect.min.y), egui::pos2(cx, rect.max.y)],
            egui::Stroke::new(style.width, style.color),
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
