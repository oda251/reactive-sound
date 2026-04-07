use rdev::{listen, Event, EventType};
use std::sync::mpsc;
use std::time::Instant;

pub struct TypingStats {
    pub wpm: f32,
    pub key_count: u64,
}

/// Spawns a background thread that captures global keyboard events
/// and sends TypingStats updates via the returned receiver.
pub fn start_keyboard_listener() -> mpsc::Receiver<TypingStats> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let (event_tx, event_rx) = mpsc::channel();

        std::thread::spawn(move || {
            listen(move |event| {
                let _ = event_tx.send(event);
            })
            .expect("failed to listen for keyboard events");
        });

        let mut timestamps: Vec<Instant> = Vec::new();
        let window_secs = 5.0;
        let mut key_count: u64 = 0;

        loop {
            match event_rx.recv() {
                Ok(Event {
                    event_type: EventType::KeyPress(_),
                    ..
                }) => {
                    let now = Instant::now();
                    key_count += 1;
                    timestamps.push(now);

                    // Remove old timestamps outside the window
                    timestamps.retain(|t| now.duration_since(*t).as_secs_f32() < window_secs);

                    // Calculate WPM (assuming average 5 chars per word)
                    let keys_in_window = timestamps.len() as f32;
                    let wpm = (keys_in_window / 5.0) * (60.0 / window_secs);

                    let _ = tx.send(TypingStats { wpm, key_count });
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });

    rx
}
