use reactive_bgm_engine::InputEvent;
use rdev::{listen, EventType};
use std::sync::mpsc;
use std::time::Instant;

/// Trait for input adapters. Each adapter sends InputEvents through a shared channel.
pub trait InputAdapter: Send + 'static {
    fn start(self, tx: mpsc::Sender<InputEvent>);
}

/// Global keyboard capture via rdev.
pub struct RdevAdapter;

impl InputAdapter for RdevAdapter {
    fn start(self, tx: mpsc::Sender<InputEvent>) {
        std::thread::spawn(move || {
            listen(move |event| match event.event_type {
                EventType::KeyPress(_) => {
                    let _ = tx.send(InputEvent::KeyPress {
                        timestamp: Instant::now(),
                    });
                }
                EventType::ButtonPress(_) => {
                    let _ = tx.send(InputEvent::MouseClick {
                        timestamp: Instant::now(),
                    });
                }
                _ => {}
            })
            .expect("failed to listen for keyboard events");
        });
    }
}

/// Captures key events from egui (when window has focus).
pub struct EguiAdapter;

impl EguiAdapter {
    pub fn drain(ctx: &eframe::egui::Context, tx: &mpsc::Sender<InputEvent>) {
        let count = ctx.input(|i| {
            i.events
                .iter()
                .filter(|e| matches!(e, eframe::egui::Event::Key { pressed: true, .. }))
                .count()
        });
        let now = Instant::now();
        for _ in 0..count {
            let _ = tx.send(InputEvent::KeyPress { timestamp: now });
        }
    }
}
