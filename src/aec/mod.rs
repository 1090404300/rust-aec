// AEC processing engine using sonora (pure Rust WebRTC AEC3 port).

use anyhow::Result;
use sonora::config::EchoCanceller;
use sonora::{AudioProcessing, Config, StreamConfig};
use std::time::{Duration, Instant};

/// Frame size in samples at 48kHz (10ms).
pub const FRAME_SIZE: usize = 480;
/// Number of audio channels.
pub const NUM_CHANNELS: usize = 1;
/// Sample rate in Hz.
pub const SAMPLE_RATE: usize = 48_000;
const DELAY_RECHECK_INTERVAL: Duration = Duration::from_secs(3);

pub struct AecProcessor {
    apm: AudioProcessing,
    render_buf: Vec<f32>,
    last_delay_recheck: Instant,
    lock_delay: bool,
    fixed_delay_ms: Option<i32>,
}

impl AecProcessor {
    pub fn new() -> Result<Self> {
        let stream_config = StreamConfig::new(SAMPLE_RATE as u32, NUM_CHANNELS as u16);
        let config = Config {
            echo_canceller: Some(EchoCanceller::default()),
            ..Default::default()
        };
        let apm = AudioProcessing::builder()
            .config(config)
            .capture_config(stream_config)
            .render_config(stream_config)
            .build();
        Ok(Self {
            apm,
            render_buf: vec![0.0f32; FRAME_SIZE],
            last_delay_recheck: Instant::now(),
            lock_delay: false,
            fixed_delay_ms: None,
        })
    }

    pub fn configure_delay_lock(&mut self, lock_delay: bool, delay_ms: Option<i32>) {
        self.lock_delay = lock_delay;
        self.fixed_delay_ms = delay_ms;
        if lock_delay {
            if let Some(delay) = delay_ms {
                let _ = self.apm.set_stream_delay_ms(delay);
            }
        } else {
            self.apm.reset_delay_estimator();
            self.last_delay_recheck = Instant::now();
        }
    }

    pub fn current_delay_ms(&self) -> Option<i32> {
        self.apm.statistics().delay_ms
    }

    /// Process one 10ms frame.
    /// `mic_frame` and `ref_frame` must each be exactly FRAME_SIZE samples.
    /// Returns processed (echo-cancelled) samples.
    pub fn process_frame(&mut self, mic_frame: &[f32], ref_frame: &[f32], out: &mut [f32]) {
        if self.lock_delay {
            if let Some(delay) = self.fixed_delay_ms {
                let _ = self.apm.set_stream_delay_ms(delay);
            }
        } else if self.last_delay_recheck.elapsed() >= DELAY_RECHECK_INTERVAL {
            self.apm.reset_delay_estimator();
            self.last_delay_recheck = Instant::now();
        }

        // Feed far-end (speaker/reference) signal.
        self.render_buf.fill(0.0);
        if let Err(e) = self
            .apm
            .process_render_f32(&[ref_frame], &mut [&mut self.render_buf])
        {
            eprintln!("[aec] process_render error: {e}");
        }

        // Process near-end (microphone) signal — echo cancellation applied here.
        if let Err(e) = self.apm.process_capture_f32(&[mic_frame], &mut [out]) {
            eprintln!("[aec] process_capture error: {e}");
            // Passthrough mic audio on error.
            out.copy_from_slice(mic_frame);
        }
    }
}
