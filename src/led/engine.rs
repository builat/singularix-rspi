/*!
    LED render engine: a dedicated thread that owns the framebuffer, the
    brightness/rainbow state and the output device. It replaces the Arduino
    firmware's loop() — commands are applied immediately, and while the
    rainbow is enabled a frame is rendered every 45 ms.
*/

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

use tokio::sync::broadcast;

use super::color::{color_hsv, gamma32};
use super::device::LedDevice;
use super::protocol::{self, LedCommand, Parsed};

const FRAME_INTERVAL: Duration = Duration::from_millis(45);
const RAINBOW_SPEED: u16 = 180;
const BOOT_BRIGHTNESS: u8 = 80;

#[derive(Clone)]
pub struct LedEngineHandle {
    tx: Sender<Parsed>,
    events: broadcast::Sender<String>,
    alive: Arc<AtomicBool>,
}

impl LedEngineHandle {
    pub fn send(&self, parsed: Parsed) {
        if let Err(e) = self.tx.send(parsed) {
            eprintln!("[LED] engine is down: {e}");
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.events.subscribe()
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }
}

/// Spawns the engine thread. `device` is the opened LED output (SPI on the
/// Pi, a no-op sink on dev machines).
pub fn spawn(device: Box<dyn LedDevice>, led_count: usize) -> LedEngineHandle {
    let (tx, rx) = channel::<Parsed>();
    let (events, _) = broadcast::channel::<String>(256);
    let alive = Arc::new(AtomicBool::new(true));
    let alive_thread = alive.clone();
    let events_thread = events.clone();

    thread::Builder::new()
        .name("led-engine".to_string())
        .spawn(move || {
            let mut state = EngineState::new(device, led_count);
            run(&mut state, &rx, &events_thread);
            alive_thread.store(false, Ordering::Relaxed);
        })
        .expect("failed to spawn led-engine thread");

    LedEngineHandle { tx, events, alive }
}

fn run(state: &mut EngineState, rx: &Receiver<Parsed>, events: &broadcast::Sender<String>) {
    state.render(); // boot: latch the strip dark, like strip.begin() + show()
    let _ = events.send("status:engine-started".to_string());
    println!(
        "[LED] engine started: {} LEDs, boot brightness {BOOT_BRIGHTNESS}",
        state.pixels.len()
    );

    let mut last_tick = Instant::now();
    loop {
        let timeout = FRAME_INTERVAL.saturating_sub(last_tick.elapsed());
        match rx.recv_timeout(timeout) {
            Ok(parsed) => state.handle(parsed, events),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
        if last_tick.elapsed() >= FRAME_INTERVAL {
            last_tick = Instant::now();
            if state.rainbow {
                state.rainbow_frame();
            }
        }
    }
    let _ = events.send("status:engine-stopped".to_string());
}

/// All mutable LED state. Kept separate from the thread plumbing so the
/// firmware behavior can be unit-tested with a NullDevice.
pub struct EngineState {
    device: Box<dyn LedDevice>,
    /// Raw (pre-brightness) colors, one [r, g, b] per LED.
    pixels: Vec<[u8; 3]>,
    /// Brightness-scaled frame handed to the device.
    out: Vec<[u8; 3]>,
    brightness: u8,
    rainbow: bool,
    rainbow_offset: u16,
}

impl EngineState {
    pub fn new(device: Box<dyn LedDevice>, led_count: usize) -> Self {
        EngineState {
            device,
            pixels: vec![[0, 0, 0]; led_count],
            out: vec![[0, 0, 0]; led_count],
            brightness: BOOT_BRIGHTNESS,
            rainbow: false,
            rainbow_offset: 0,
        }
    }

    /// One command: apply the firmware's stop-rainbow rule, run the command,
    /// broadcast the ACK.
    pub fn handle(&mut self, parsed: Parsed, events: &broadcast::Sender<String>) {
        if parsed.stops_rainbow {
            self.rainbow = false;
        }
        if let Some(cmd) = parsed.command {
            self.apply(&cmd);
        }
        let _ = events.send(protocol::ack_json(&parsed));
    }

    fn apply(&mut self, cmd: &LedCommand) {
        match cmd {
            LedCommand::Off => {
                self.pixels.fill([0, 0, 0]);
                self.render();
            }
            LedCommand::Rainbow => {
                // No immediate frame — the first one renders on the next
                // tick, like the firmware's processRainbowNonBlocking().
                self.rainbow = true;
            }
            LedCommand::Range {
                start,
                end,
                r,
                g,
                b,
            } => {
                let (mut s, mut e) = (*start, *end);
                if s > e {
                    std::mem::swap(&mut s, &mut e);
                }
                if s < 0 {
                    s = 0;
                }
                let last = self.pixels.len() as i32 - 1;
                if e > last {
                    e = last;
                }
                let color = [*r, *g, *b];
                for i in s..=e {
                    self.pixels[i as usize] = color;
                }
                self.render();
            }
            LedCommand::Color { r, g, b } => {
                let color = [*r, *g, *b];
                self.pixels.fill(color);
                self.render();
            }
            LedCommand::Brightness { value } => {
                self.brightness = *value;
                self.render();
            }
            LedCommand::Noop => {}
        }
    }

    /// Port of processRainbowNonBlocking's frame body.
    fn rainbow_frame(&mut self) {
        let n = self.pixels.len();
        if n == 0 {
            return;
        }
        for i in 0..n {
            let hue =
                (self.rainbow_offset as u32).wrapping_add((i as u32) * 65535 / n as u32) as u16;
            let (r, g, b) = gamma32(color_hsv(hue));
            self.pixels[i] = [r, g, b];
        }
        self.render();
        self.rainbow_offset = self.rainbow_offset.wrapping_add(RAINBOW_SPEED);
    }

    /// Writes the framebuffer to the device, scaled by the global
    /// brightness. The scaling matches Adafruit_NeoPixel's setPixelColor
    /// `(c * brightness) >> 8` with the stored value = brightness + 1
    /// (0 means "no scaling"); unlike the library's in-place rescale this
    /// keeps the raw colors, so raising brightness back restores them.
    fn render(&mut self) {
        let stored = self.brightness.wrapping_add(1);
        for (o, px) in self.out.iter_mut().zip(&self.pixels) {
            *o = if stored == 0 {
                *px
            } else {
                [
                    ((px[0] as u16 * stored as u16) >> 8) as u8,
                    ((px[1] as u16 * stored as u16) >> 8) as u8,
                    ((px[2] as u16 * stored as u16) >> 8) as u8,
                ]
            };
        }
        if let Err(e) = self.device.render(&self.out) {
            eprintln!("[LED] render failed: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{BOOT_BRIGHTNESS, EngineState};
    use crate::led::color::{color_hsv, gamma32};
    use crate::led::device::NullDevice;
    use crate::led::protocol::{self, LedCommand};

    /// Polls the SSE channel until a message containing `pattern` shows up
    /// (the engine thread races with the test).
    fn wait_for_event(sse: &mut tokio::sync::broadcast::Receiver<String>, pattern: &str) -> String {
        for _ in 0..200 {
            match sse.try_recv() {
                Ok(msg) if msg.contains(pattern) => return msg,
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => panic!("SSE channel error: {e}"),
            }
        }
        panic!("no SSE event containing {pattern:?} arrived in time");
    }

    fn state(leds: usize) -> EngineState {
        EngineState::new(Box::new(NullDevice::new()), leds)
    }

    fn device(state: &mut EngineState) -> &NullDevice {
        state.device.as_any().downcast_ref::<NullDevice>().unwrap()
    }

    fn apply(state: &mut EngineState, cmd: LedCommand) {
        state.apply(&cmd);
    }

    #[test]
    fn boot_state_is_dark_at_brightness_80() {
        let mut s = state(4);
        assert_eq!(s.brightness, BOOT_BRIGHTNESS);
        assert!(s.pixels.iter().all(|p| *p == [0, 0, 0]));
        s.render();
        assert_eq!(device(&mut s).frames, 1);
        assert!(device(&mut s).last_frame.iter().all(|p| *p == [0, 0, 0]));
    }

    #[test]
    fn brightness_scales_the_frame() {
        let mut s = state(2);
        apply(
            &mut s,
            LedCommand::Color {
                r: 255,
                g: 255,
                b: 255,
            },
        );
        // 255 * (80 + 1) >> 8 = 80
        assert_eq!(device(&mut s).last_frame[0], [80, 80, 80]);

        apply(&mut s, LedCommand::Brightness { value: 255 });
        assert_eq!(device(&mut s).last_frame[0], [255, 255, 255]);

        // Brightness 0 goes dark but the framebuffer keeps its colors.
        apply(&mut s, LedCommand::Brightness { value: 0 });
        assert_eq!(device(&mut s).last_frame[0], [0, 0, 0]);
        assert_eq!(s.pixels[0], [255, 255, 255]);
        apply(&mut s, LedCommand::Brightness { value: 80 });
        assert_eq!(device(&mut s).last_frame[0], [80, 80, 80]);
    }

    #[test]
    fn range_swaps_and_clamps_like_the_firmware() {
        let mut s = state(8);
        apply(
            &mut s,
            LedCommand::Range {
                start: 6,
                end: 2,
                r: 10,
                g: 0,
                b: 0,
            },
        );
        for (i, px) in s.pixels.iter().enumerate() {
            let painted = (2..=6).contains(&i);
            assert_eq!(
                *px,
                if painted { [10, 0, 0] } else { [0, 0, 0] },
                "pixel {i}"
            );
        }

        apply(
            &mut s,
            LedCommand::Range {
                start: -5,
                end: 1,
                r: 0,
                g: 20,
                b: 0,
            },
        );
        assert_eq!(s.pixels[0], [0, 20, 0]);
        assert_eq!(s.pixels[1], [0, 20, 0]);
        assert_eq!(s.pixels[2], [10, 0, 0]); // untouched by the clamped range

        apply(
            &mut s,
            LedCommand::Range {
                start: 0,
                end: 99,
                r: 0,
                g: 0,
                b: 30,
            },
        );
        assert!(s.pixels.iter().all(|p| *p == [0, 0, 30]));

        // Fully negative range paints nothing, exactly like the C loop.
        apply(
            &mut s,
            LedCommand::Range {
                start: -9,
                end: -2,
                r: 9,
                g: 9,
                b: 9,
            },
        );
        assert!(s.pixels.iter().all(|p| *p == [0, 0, 30]));
    }

    #[test]
    fn off_and_noop() {
        let mut s = state(3);
        apply(&mut s, LedCommand::Color { r: 1, g: 2, b: 3 });
        apply(&mut s, LedCommand::Off);
        assert!(s.pixels.iter().all(|p| *p == [0, 0, 0]));

        let frames_before = device(&mut s).frames;
        apply(&mut s, LedCommand::Noop);
        assert_eq!(device(&mut s).frames, frames_before);
    }

    #[test]
    fn rainbow_rule_any_recognized_command_stops_it() {
        let (events, _) = tokio::sync::broadcast::channel(16);
        let mut s = state(4);

        s.handle(protocol::rainbow(), &events);
        assert!(s.rainbow);

        // Failing command with a valid id: stops the rainbow.
        s.handle(protocol::color(999, 0, 0), &events);
        assert!(!s.rainbow);

        // Unparseable input or a bad id: rainbow untouched (early returns
        // happen before the firmware clears rainbowEnabled).
        s.handle(protocol::rainbow(), &events);
        s.handle(protocol::parse_ascii("garbage"), &events);
        assert!(s.rainbow);
        s.handle(protocol::parse_ascii("1000"), &events);
        assert!(s.rainbow);
    }

    #[test]
    fn rainbow_frame_math_and_offset_advance() {
        let mut s = state(16);
        apply(&mut s, LedCommand::Rainbow);
        s.rainbow_offset = 1000;
        s.rainbow_frame();

        let expected: Vec<[u8; 3]> = (0..16)
            .map(|i| {
                let hue = (1000u32 + i * 65535 / 16) as u16;
                let (r, g, b) = gamma32(color_hsv(hue));
                [r, g, b]
            })
            .collect();
        assert_eq!(s.pixels, expected);
        assert_eq!(s.rainbow_offset, 1000 + 180);

        // Offset wraps and is not reset when the rainbow restarts.
        s.rainbow = false;
        s.rainbow_offset = 65500;
        s.rainbow_frame();
        assert_eq!(s.rainbow_offset, (65500u16).wrapping_add(180));
    }

    #[test]
    fn acks_are_broadcast_per_command() {
        let (events, _) = tokio::sync::broadcast::channel(16);
        let mut rx_events = events.subscribe();
        let mut s = state(2);

        s.handle(protocol::rainbow(), &events);
        let msg = rx_events.try_recv().unwrap();
        assert!(msg.contains("\"command\":\"rainbow\""));
        assert!(msg.contains("\"status\":\"ok\""));

        s.handle(protocol::parse_ascii("4 999 0 0"), &events);
        let msg = rx_events.try_recv().unwrap();
        assert!(msg.contains("\"status\":\"err\""));
        assert!(msg.contains("\"detail\":\"range\""));
        assert!(msg.contains("\"command_code\":4"));
    }

    #[test]
    fn spawned_engine_applies_commands_and_shuts_down() {
        let handle = super::spawn(Box::new(NullDevice::new()), 4);
        assert!(handle.is_alive());
        let mut sse = handle.subscribe();
        let mut shutdown = handle.subscribe();
        handle.send(protocol::color(9, 8, 7));
        let msg = wait_for_event(&mut sse, "\"command\":\"set rgb\"");
        assert!(msg.contains("\"status\":\"ok\""));

        // Dropping every handle drops the sender; the thread exits and
        // reports it on the SSE channel.
        drop(handle);
        wait_for_event(&mut shutdown, "status:engine-stopped");
    }
}
