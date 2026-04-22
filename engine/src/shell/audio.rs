use anyhow::{bail, Context};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::core::dsp::DspProcessor;
use crate::core::{EngineConfig, Scheduler};
use crate::shell::bridge::Bridge;
use crate::shell::command::Command;

const CMD_RING_CAPACITY: usize = 256;

pub struct AudioOutput {
    _stream: Stream,
    cmd_tx: rtrb::Producer<Command>,
    playhead: Arc<AtomicU64>,
    start_time: std::time::Instant,
}

impl AudioOutput {
    pub fn start(config: &EngineConfig) -> anyhow::Result<Self> {
        let device = resolve_device(config)?;
        let stream_config = device
            .default_output_config()
            .context("failed to get default output config")?;

        let sample_rate = config.sample_rate_or(stream_config.sample_rate().0);
        let dsp = Box::new(DspProcessor::new(sample_rate, 128));
        let scheduler = Scheduler::new(sample_rate);

        let (cmd_tx, cmd_rx) = rtrb::RingBuffer::new(CMD_RING_CAPACITY);

        let playhead = Arc::new(AtomicU64::new(0));
        let playhead_writer = playhead.clone();

        let bridge = Bridge::new(scheduler, dsp, cmd_rx, config);

        let stream = match stream_config.sample_format() {
            SampleFormat::F32 => {
                build_stream_f32(&device, &stream_config.into(), bridge, playhead_writer)?
            }
            SampleFormat::I16 => build_stream_convert::<i16>(
                &device,
                &stream_config.into(),
                bridge,
                playhead_writer,
            )?,
            fmt => bail!("unsupported sample format: {fmt:?}"),
        };

        stream.play().context("failed to start audio stream")?;
        let start_time = std::time::Instant::now();
        Ok(Self {
            _stream: stream,
            cmd_tx,
            playhead,
            start_time,
        })
    }

    pub fn send(&mut self, cmd: Command) -> Result<(), rtrb::PushError<Command>> {
        self.cmd_tx.push(cmd)
    }

    pub fn playhead(&self) -> f32 {
        f32::from_bits(self.playhead.load(Ordering::Relaxed) as u32)
    }

    pub fn start_time(&self) -> std::time::Instant {
        self.start_time
    }
}

fn resolve_device(config: &EngineConfig) -> anyhow::Result<Device> {
    let host = cpal::default_host();

    if let Some(name) = &config.device_name {
        let devices = host
            .output_devices()
            .context("failed to enumerate output devices")?;
        for d in devices {
            if d.name()
                .map(|n| n.contains(name.as_str()))
                .unwrap_or(false)
            {
                return Ok(d);
            }
        }
        bail!("audio device not found: {name}");
    }

    host.default_output_device()
        .context("no default output device")
}

fn build_stream_f32(
    device: &Device,
    config: &StreamConfig,
    mut bridge: Bridge,
    playhead: Arc<AtomicU64>,
) -> anyhow::Result<Stream> {
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            bridge.fill(data);
            playhead.store(bridge.playhead(0).to_bits() as u64, Ordering::Relaxed);
        },
        |err| log::error!("audio stream error: {err}"),
        None,
    )?;
    Ok(stream)
}

fn build_stream_convert<
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32> + Send + 'static,
>(
    device: &Device,
    config: &StreamConfig,
    mut bridge: Bridge,
    playhead: Arc<AtomicU64>,
) -> anyhow::Result<Stream> {
    let mut convert_buf: Vec<f32> = Vec::new();

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            convert_buf.resize(data.len(), 0.0);
            bridge.fill(&mut convert_buf);
            for (i, sample) in data.iter_mut().enumerate() {
                *sample = T::from_sample(convert_buf[i]);
            }
            playhead.store(bridge.playhead(0).to_bits() as u64, Ordering::Relaxed);
        },
        |err| log::error!("audio stream error: {err}"),
        None,
    )?;
    Ok(stream)
}
