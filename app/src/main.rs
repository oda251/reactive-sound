mod input;
mod raw_rhythm_provider;

use eframe::egui;
use reactive_bgm_engine::{
    AccumulativeEffect, Engine, ImmediateEffect, InputEffect, InputEvent,
};
use std::sync::mpsc;
use std::time::Instant;

use raw_rhythm_provider::{KeyClickEffect, RhythmAccumulator, MEASURES};

fn main() {
    env_logger::init();

    let engine = Engine::start_default().expect("failed to start engine");

    let (event_tx, event_rx) = mpsc::channel::<InputEvent>();

    input::start_rdev_adapter(event_tx.clone());
    let egui_tx = event_tx;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 420.0])
            .with_title("Reactive BGM"),
        ..Default::default()
    };

    eframe::run_native(
        "Reactive BGM",
        options,
        Box::new(move |_cc| Ok(Box::new(App::new(engine, egui_tx, event_rx)))),
    )
    .expect("failed to run eframe");
}

const UPDATE_INTERVAL_MS: u128 = 2000;

struct App {
    engine: Engine,
    egui_tx: mpsc::Sender<InputEvent>,
    event_rx: mpsc::Receiver<InputEvent>,
    accumulator: RhythmAccumulator,
    click: KeyClickEffect,
    key_count: u64,
    last_update: Option<Instant>,
    note_positions: Vec<f32>,
}

impl App {
    fn new(
        engine: Engine,
        egui_tx: mpsc::Sender<InputEvent>,
        event_rx: mpsc::Receiver<InputEvent>,
    ) -> Self {
        Self {
            engine,
            egui_tx,
            event_rx,
            accumulator: RhythmAccumulator::new(),
            click: KeyClickEffect::new(),
            key_count: 0,
            last_update: None,
            note_positions: Vec::new(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        input::drain_egui_keys(ui.ctx(), &self.egui_tx);

        let now = Instant::now();

        while let Ok(event) = self.event_rx.try_recv() {
            self.key_count += 1;
            self.accumulator.on_event(&event, now);
            self.click.on_event(&event, now);
        }

        // Immediate effects → Engine
        for action in self.click.drain_actions() {
            let _ = self.engine.send_immediate(action);
        }

        // Accumulative effects → Engine (periodic)
        let should_update = match self.last_update {
            Some(last) => now.duration_since(last).as_millis() >= UPDATE_INTERVAL_MS,
            None => true,
        };

        if should_update {
            let pattern = self.accumulator.score(now);
            self.note_positions = self.accumulator.note_positions(now);
            let _ = self.engine.set_pattern(0, pattern);
            self.last_update = Some(now);
        }

        let playhead = self.engine.playhead();
        let current_measure = ((playhead * MEASURES as f32) as usize).min(MEASURES - 1);
        let measure_playhead = (playhead * MEASURES as f32).fract();

        let mut measure_events: Vec<Vec<f32>> = vec![Vec::new(); MEASURES];
        for &pos in &self.note_positions {
            let m = ((pos * MEASURES as f32) as usize).min(MEASURES - 1);
            let pos_in_m = (pos * MEASURES as f32).fract();
            measure_events[m].push(pos_in_m);
        }

        // --- UI ---
        ui.heading("Reactive BGM");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Keys:");
            ui.strong(format!("{}", self.key_count));
            ui.label("  Events:");
            ui.strong(format!("{}", self.accumulator.event_count()));
        });

        ui.separator();
        ui.label("Type anywhere — your rhythm becomes music");
        ui.add_space(4.0);

        let lane_width = ui.available_width() - 30.0;
        let lane_height = 20.0;
        let lane_spacing = 2.0;

        for m in 0..MEASURES {
            let is_playing = m == current_measure;

            ui.horizontal(|ui| {
                ui.label(format!("{:>2}", m + 1));

                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(lane_width, lane_height),
                    egui::Sense::hover(),
                );

                let bg = if is_playing {
                    egui::Color32::from_gray(35)
                } else {
                    egui::Color32::from_gray(25)
                };
                ui.painter().rect_filled(rect, 2.0, bg);

                for beat in 1..4 {
                    let x = rect.min.x + lane_width * (beat as f32 / 4.0);
                    ui.painter().line_segment(
                        [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                        egui::Stroke::new(1.0, egui::Color32::from_gray(45)),
                    );
                }

                for &event_pos in &measure_events[m] {
                    let x = rect.min.x + lane_width * event_pos;
                    let note_rect = egui::Rect::from_min_size(
                        egui::pos2(x - 1.5, rect.min.y + 2.0),
                        egui::vec2(3.0, lane_height - 4.0),
                    );

                    let played = if m < current_measure {
                        true
                    } else if m == current_measure {
                        event_pos < measure_playhead
                    } else {
                        false
                    };

                    let color = if played {
                        egui::Color32::from_rgb(30, 80, 30)
                    } else {
                        egui::Color32::from_rgb(60, 220, 60)
                    };
                    ui.painter().rect_filled(note_rect, 1.0, color);
                }

                if is_playing {
                    let ph_x = rect.min.x + lane_width * measure_playhead;
                    ui.painter().line_segment(
                        [egui::pos2(ph_x, rect.min.y), egui::pos2(ph_x, rect.max.y)],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 80, 80)),
                    );
                }
            });

            if m < MEASURES - 1 {
                ui.add_space(lane_spacing);
            }
        }

        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(16));
    }
}
