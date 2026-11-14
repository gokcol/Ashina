//! Local microphone/playback streams. No device is opened until an explicit action.
use anyhow::{Result, anyhow, bail};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use mypdf_milestone0::voice::{Clip, MAX_SAMPLES, RATE};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

pub fn permission(request: bool) -> i32 {
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn mypdf_microphone_permission(request: i32) -> i32;
        }
        unsafe { mypdf_microphone_permission(i32::from(request)) }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = request;
        3
    }
}
#[derive(Default)]
pub struct State {
    pub target: Option<(u64, String)>,
    pub pending_permission: bool,
    pub pending_clip: Option<(u64, String, Clip)>,
    pub recorder: Option<Recorder>,
    pub player: Option<Player>,
    pub error: Option<String>,
}
pub struct Recorder {
    _stream: cpal::Stream,
    samples: Arc<Mutex<Vec<i16>>>,
    paused: Arc<AtomicBool>,
    error: Arc<Mutex<Option<String>>>,
}
impl Recorder {
    pub fn start() -> Result<Self> {
        let device = cpal::default_host()
            .default_input_device()
            .ok_or_else(|| anyhow!("No microphone found. Connect a microphone and try again."))?;
        let supported = device.default_input_config()?;
        let config: cpal::StreamConfig = supported.clone().into();
        let samples = Arc::new(Mutex::new(Vec::with_capacity(MAX_SAMPLES)));
        let paused = Arc::new(AtomicBool::new(false));
        let error = Arc::new(Mutex::new(None));
        macro_rules! input {
            ($t:ty) => {
                input::<$t>(
                    &device,
                    &config,
                    samples.clone(),
                    paused.clone(),
                    error.clone(),
                )?
            };
        }
        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => input!(f32),
            cpal::SampleFormat::F64 => input!(f64),
            cpal::SampleFormat::I16 => input!(i16),
            cpal::SampleFormat::U16 => input!(u16),
            cpal::SampleFormat::I32 => input!(i32),
            cpal::SampleFormat::U32 => input!(u32),
            other => bail!("Microphone sample format {other} is not supported"),
        };
        stream.play()?;
        Ok(Self {
            _stream: stream,
            samples,
            paused,
            error,
        })
    }
    pub fn paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }
    pub fn toggle_pause(&self) {
        self.paused.store(!self.paused(), Ordering::Relaxed);
    }
    pub fn duration(&self) -> f32 {
        self.samples.lock().unwrap().len() as f32 / RATE as f32
    }
    pub fn full(&self) -> bool {
        self.samples.lock().unwrap().len() >= MAX_SAMPLES
    }
    pub fn error(&self) -> Option<String> {
        self.error.lock().unwrap().clone()
    }
    pub fn finish(self) -> Result<Clip> {
        self.paused.store(true, Ordering::Relaxed);
        drop(self._stream);
        Clip::from_samples(&self.samples.lock().unwrap())
    }
}
fn input<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    samples: Arc<Mutex<Vec<i16>>>,
    paused: Arc<AtomicBool>,
    error: Arc<Mutex<Option<String>>>,
) -> Result<cpal::Stream>
where
    T: cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels as usize;
    let ratio = f64::from(RATE) / f64::from(config.sample_rate.0);
    let mut phase = 0.;
    Ok(device.build_input_stream(
        config,
        move |data: &[T], _| {
            if paused.load(Ordering::Relaxed) {
                return;
            }
            let mut samples = samples.lock().unwrap();
            for frame in data.chunks_exact(channels) {
                phase += ratio;
                while phase >= 1. && samples.len() < MAX_SAMPLES {
                    let sample: f32 =
                        frame.iter().map(|s| s.to_sample::<f32>()).sum::<f32>() / channels as f32;
                    samples.push((sample.clamp(-1., 1.) * 32767.).round() as i16);
                    phase -= 1.;
                }
                if samples.len() >= MAX_SAMPLES {
                    break;
                }
            }
        },
        move |e| {
            *error.lock().unwrap() = Some(e.to_string());
        },
        None,
    )?)
}
pub struct Player {
    _stream: cpal::Stream,
    pub paused: Arc<AtomicBool>,
    pub position: Arc<AtomicUsize>,
    length: usize,
    error: Arc<Mutex<Option<String>>>,
}
impl Player {
    pub fn start(clip: &Clip) -> Result<Self> {
        let samples = Arc::new(clip.samples()?);
        if samples.is_empty() {
            bail!("This voice note has no recording yet");
        }
        let length = samples.len();
        let device = cpal::default_host()
            .default_output_device()
            .ok_or_else(|| anyhow!("No audio output device found"))?;
        let supported = device.default_output_config()?;
        let config: cpal::StreamConfig = supported.clone().into();
        let paused = Arc::new(AtomicBool::new(false));
        let position = Arc::new(AtomicUsize::new(0));
        let error = Arc::new(Mutex::new(None));
        macro_rules! output {
            ($t:ty) => {
                output::<$t>(
                    &device,
                    &config,
                    samples,
                    paused.clone(),
                    position.clone(),
                    error.clone(),
                )?
            };
        }
        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => output!(f32),
            cpal::SampleFormat::F64 => output!(f64),
            cpal::SampleFormat::I16 => output!(i16),
            cpal::SampleFormat::U16 => output!(u16),
            cpal::SampleFormat::I32 => output!(i32),
            cpal::SampleFormat::U32 => output!(u32),
            other => bail!("Audio output format {other} is not supported"),
        };
        stream.play()?;
        Ok(Self {
            _stream: stream,
            paused,
            position,
            length,
            error,
        })
    }
    pub fn done(&self) -> bool {
        self.position.load(Ordering::Relaxed) >= self.length
    }
    pub fn seconds(&self) -> f32 {
        self.position.load(Ordering::Relaxed) as f32 / RATE as f32
    }
    pub fn error(&self) -> Option<String> {
        self.error.lock().unwrap().clone()
    }
    pub fn toggle_pause(&self) {
        self.paused.fetch_xor(true, Ordering::Relaxed);
    }
}
fn output<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    samples: Arc<Vec<i16>>,
    paused: Arc<AtomicBool>,
    position: Arc<AtomicUsize>,
    error: Arc<Mutex<Option<String>>>,
) -> Result<cpal::Stream>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let channels = config.channels as usize;
    let step = f64::from(RATE) / f64::from(config.sample_rate.0);
    let mut cursor = 0.;
    Ok(device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            for frame in data.chunks_mut(channels) {
                let mut sample = 0.;
                if !paused.load(Ordering::Relaxed) && (cursor as usize) < samples.len() {
                    let index = cursor as usize;
                    let fraction = (cursor - index as f64) as f32;
                    sample = (samples[index] as f32 * (1. - fraction)
                        + samples.get(index + 1).copied().unwrap_or(samples[index]) as f32
                            * fraction)
                        / 32768.;
                    cursor += step;
                }
                frame.fill(T::from_sample(sample));
            }
            position.store((cursor as usize).min(samples.len()), Ordering::Relaxed);
        },
        move |e| {
            *error.lock().unwrap() = Some(e.to_string());
        },
        None,
    )?)
}
