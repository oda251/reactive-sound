mod input;
mod mapper;

use eframe::egui;
use reactive_bgm_engine::{Engine, PARAM_FREQ, PARAM_GAIN, PARAM_GATE};
use std::sync::mpsc;

use input::{start_keyboard_listener, TypingStats};
use mapper::Mapper;

fn main() {
    env_logger::init();

    let engine = Engine::start_default().expect("failed to start engine");

    engine.update_pattern("o: sin 220 >> mul 0.05").expect("pattern");

    let kb_rx = start_keyboard_listener();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_title("Reactive BGM"),
        ..Default::default()
    };

    eframe::run_native(
        "Reactive BGM",
        options,
        Box::new(move |_cc| Ok(Box::new(App::new(engine, kb_rx)))),
    )
    .expect("failed to run eframe");
}

struct App {
    engine: Engine,
    kb_rx: mpsc::Receiver<TypingStats>,
    mapper: Mapper,
    wpm: f32,
    key_count: u64,
    tier_label: &'static str,
}

impl App {
    fn new(engine: Engine, kb_rx: mpsc::Receiver<TypingStats>) -> Self {
        Self {
            engine,
            kb_rx,
            mapper: Mapper::new(),
            wpm: 0.0,
            key_count: 0,
            tier_label: "Idle",
        }
    }

    fn apply_params(&self, params: &mapper::MappedParams) {
        let _ = self.engine.update_pattern(params.pattern);
        let _ = self.engine.set_synth_param(PARAM_FREQ, params.freq);
        let _ = self.engine.set_synth_param(PARAM_GAIN, params.gain);
        let _ = self.engine.set_synth_param(PARAM_GATE, if params.gate { 1.0 } else { 0.0 });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Drain keyboard events
        while let Ok(stats) = self.kb_rx.try_recv() {
            self.wpm = stats.wpm;
            self.key_count = stats.key_count;

            if let Some(params) = self.mapper.update(self.wpm) {
                self.tier_label = params.label;
                eprintln!("[{:.0} WPM] Tier changed → {}", self.wpm, self.tier_label);
                self.apply_params(&params);
            }
        }

        ui.heading("Reactive BGM");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("WPM:");
            ui.strong(format!("{:.0}", self.wpm));
        });

        ui.horizontal(|ui| {
            ui.label("Keys:");
            ui.strong(format!("{}", self.key_count));
        });

        ui.horizontal(|ui| {
            ui.label("Tier:");
            ui.strong(self.tier_label);
        });

        ui.separator();

        // WPM bar
        let bar_width = (self.wpm / 120.0).clamp(0.0, 1.0);
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 24.0),
            egui::Sense::hover(),
        );
        ui.painter().rect_filled(
            rect,
            4.0,
            egui::Color32::from_gray(60),
        );
        let filled = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(rect.width() * bar_width, rect.height()),
        );
        let color = match self.tier_label {
            "Idle" => egui::Color32::from_rgb(80, 80, 120),
            "Slow" => egui::Color32::from_rgb(80, 120, 80),
            "Medium" => egui::Color32::from_rgb(180, 180, 60),
            _ => egui::Color32::from_rgb(200, 80, 60),
        };
        ui.painter().rect_filled(filled, 4.0, color);

        ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
    }
}
