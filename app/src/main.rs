mod input;
mod lane;
mod raw_rhythm_provider;

use eframe::egui;
use reactive_bgm_engine::{AccumulativeEffect, Engine, ImmediateEffect, InputEvent};
use std::sync::mpsc;
use std::time::Instant;

use input::{EguiAdapter, InputAdapter, RdevAdapter};
use lane::{draw_lane, split_into_measures, CursorStyle, LaneConfig};
use raw_rhythm_provider::{KeyClickEffect, RhythmAccumulator, MEASURES};

fn main() {
    env_logger::init();

    let engine = Engine::start_default().expect("failed to start engine");

    let (event_tx, event_rx) = mpsc::channel::<InputEvent>();

    // Input adapters — add more here (e.g., MidiAdapter, OscAdapter)
    RdevAdapter.start(event_tx.clone());

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
    immediate_effects: Vec<Box<dyn ImmediateEffect>>,
    accumulative_effects: Vec<Box<dyn AccumulativeEffect>>,
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
            immediate_effects: vec![Box::new(KeyClickEffect::new())],
            accumulative_effects: vec![Box::new(RhythmAccumulator::new(epoch))],
            key_count: 0,
            last_update: None,
            committed_positions: Vec::new(),
            cached_score_events: vec![Vec::new(); MEASURES],
        }
    }

    fn accumulator(&self) -> &RhythmAccumulator {
        // The first accumulative effect is always the rhythm accumulator
        self.accumulative_effects[0]
            .as_any()
            .downcast_ref::<RhythmAccumulator>()
            .expect("first accumulative effect must be RhythmAccumulator")
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        EguiAdapter::drain(ui.ctx(), &self.egui_tx);

        let now = Instant::now();

        while let Ok(event) = self.event_rx.try_recv() {
            self.key_count += 1;
            for eff in &mut self.immediate_effects {
                eff.on_event(&event, now);
            }
            for eff in &mut self.accumulative_effects {
                eff.on_event(&event, now);
            }
        }

        // Drain immediate actions
        for eff in &mut self.immediate_effects {
            for action in eff.drain_actions() {
                let _ = self.engine.send_immediate(action);
            }
        }

        // Update accumulative patterns periodically
        let should_update = match self.last_update {
            Some(last) => now.duration_since(last).as_millis() >= UPDATE_INTERVAL_MS,
            None => true,
        };

        if should_update {
            for (i, eff) in self.accumulative_effects.iter().enumerate() {
                let pattern = eff.score(now);
                let _ = self.engine.set_pattern(i, pattern);
            }
            // GUI uses the first accumulator for display
            self.committed_positions = self.accumulator().note_positions(now);
            self.cached_score_events = split_into_measures(&self.committed_positions, MEASURES);
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
            ui.strong(format!("{}", self.accumulator().event_count()));
        });

        // === Score ===
        ui.separator();
        ui.label("Score");
        ui.add_space(2.0);

        let score_note_color = |_pos: f32| egui::Color32::from_rgb(60, 220, 60);

        for m in 0..MEASURES {
            let is_active = m == playhead_measure;
            let config = LaneConfig::new(lane_width, 16.0)
                .bg(if is_active {
                    egui::Color32::from_gray(32)
                } else {
                    egui::Color32::from_gray(25)
                })
                .notes(&score_note_color);

            let config = if is_active {
                config.cursor(
                    playhead_in_measure,
                    CursorStyle {
                        color: egui::Color32::from_rgb(255, 80, 80),
                        width: 2.0,
                    },
                )
            } else {
                config
            };

            ui.horizontal(|ui| {
                ui.label(format!("{:>2}", m + 1));
                draw_lane(ui, &config, &self.cached_score_events[m]);
            });
        }

        // === Input ===
        ui.add_space(8.0);
        ui.separator();
        ui.label("Input (1 measure)");
        ui.add_space(2.0);

        let input_cursor_pos = self.accumulator().input_cursor(now);
        let measure_notes = self.accumulator().current_measure_notes(now);

        let input_note_color = |_pos: f32| egui::Color32::from_rgb(80, 180, 255);

        let input_config = LaneConfig::new(lane_width, 24.0)
            .bg(egui::Color32::from_gray(30))
            .notes(&input_note_color)
            .cursor(
                input_cursor_pos,
                CursorStyle {
                    color: egui::Color32::from_rgb(80, 200, 255),
                    width: 2.0,
                },
            );

        ui.horizontal(|ui| {
            ui.label("  ");
            draw_lane(ui, &input_config, &measure_notes);
        });

        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(16));
    }
}
