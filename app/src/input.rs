use reactive_bgm_engine::InputEvent;
use rdev::{listen, Event, EventType};
use std::sync::mpsc;
use std::time::Instant;

/// Spawns a background thread that captures global keyboard events
/// and sends InputEvents via the provided sender.
pub fn start_rdev_adapter(tx: mpsc::Sender<InputEvent>) {
    std::thread::spawn(move || {
        listen(move |event| {
            if let Event {
                event_type: EventType::KeyPress(_),
                ..
            } = event
            {
                let _ = tx.send(InputEvent::KeyPress {
                    timestamp: Instant::now(),
                });
            }
        })
        .expect("failed to listen for keyboard events");
    });
}

/// Drains egui key events and sends them as InputEvents.
pub fn drain_egui_keys(ctx: &eframe::egui::Context, tx: &mpsc::Sender<InputEvent>) {
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
