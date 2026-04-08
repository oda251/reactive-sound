mod input;
mod lane;
mod raw_rhythm_provider;

use eframe::egui;
use reactive_bgm_engine::{AccumulativeEffect, Engine, ImmediateEffect, InputEffect, InputEvent};
use std::sync::mpsc;
use std::time::Instant;

use lane::{draw_lane, split_into_measures, LaneCursor};

use raw_rhythm_provider::{KeyClickEffect, RhythmAccumulator, MEASURES};

fn main() {
    env_logger::init();

    let engine = Engine::start_default().expect("failed to start engine");

    let (event_tx, event_rx) = mpsc::channel::<InputEvent>();

    input::start_rdev_adapter(event_tx.clone());
    let egui_tx = event_tx;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 580.0])
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
    committed_positions: Vec<f32>,
    cached_score_events: Vec<Vec<f32>>,
}

impl App {
    fn new(
        engine: Engine,
        egui_tx: mpsc::Sender<InputEvent>,
        event_rx: mpsc::Receiver<InputEvent>,
    ) -> Self {
        let epoch = engine.start_time();
        Self {
            engine,
            egui_tx,
            event_rx,
            accumulator: RhythmAccumulator::new(epoch),
            click: KeyClickEffect::new(),
            key_count: 0,
            last_update: None,
            committed_positions: Vec::new(),
            cached_score_events: vec![Vec::new(); MEASURES],
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

        for action in self.click.drain_actions() {
            let _ = self.engine.send_immediate(action);
        }

        let should_update = match self.last_update {
            Some(last) => now.duration_since(last).as_millis() >= UPDATE_INTERVAL_MS,
            None => true,
        };

        if should_update {
            let pattern = self.accumulator.score(now);
            self.committed_positions = self.accumulator.note_positions(now);
            self.cached_score_events = split_into_measures(&self.committed_positions, MEASURES);
            let _ = self.engine.set_pattern(0, pattern);
            self.last_update = Some(now);
        }

        let playhead = self.engine.playhead();
        let playhead_measure = ((playhead * MEASURES as f32) as usize).min(MEASURES - 1);
        let playhead_in_measure = (playhead * MEASURES as f32).fract();

        let lane_width = ui.available_width() - 30.0;

        // --- UI ---
        ui.heading("Reactive BGM");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Keys:");
            ui.strong(format!("{}", self.key_count));
            ui.label("  Events:");
            ui.strong(format!("{}", self.accumulator.event_count()));
        });

        // === Score ===
        ui.separator();
        ui.label("Score");
        ui.add_space(2.0);

        for m in 0..MEASURES {
            let cursor = if m == playhead_measure {
                Some(LaneCursor {
                    position: playhead_in_measure,
                    color: egui::Color32::from_rgb(255, 80, 80),
                })
            } else {
                None
            };

            ui.horizontal(|ui| {
                ui.label(format!("{:>2}", m + 1));
                draw_lane(
                    ui,
                    lane_width,
                    16.0,
                    &self.cached_score_events[m],
                    egui::Color32::from_rgb(60, 220, 60),
                    cursor.as_ref(),
                    m == playhead_measure,
                );
            });
        }

        // === Input ===
        ui.add_space(8.0);
        ui.separator();
        ui.label("Input (1 measure)");
        ui.add_space(2.0);

        let input_cursor_pos = self.accumulator.input_cursor(now);
        let measure_notes = self.accumulator.current_measure_notes(now);

        let cursor = LaneCursor {
            position: input_cursor_pos,
            color: egui::Color32::from_rgb(80, 200, 255),
        };

        ui.horizontal(|ui| {
            ui.label("  ");
            draw_lane(
                ui,
                lane_width,
                24.0,
                &measure_notes,
                egui::Color32::from_rgb(80, 180, 255),
                Some(&cursor),
                true,
            );
        });

        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(16));
    }
}
