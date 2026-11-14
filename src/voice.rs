//! Portable voice-note payload. Canonical mono 16-bit PCM WAV, bounded to three minutes.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
pub const RATE: u32 = 16_000;
pub const MAX_SECONDS: usize = 180;
pub const MAX_SAMPLES: usize = RATE as usize * MAX_SECONDS;
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Clip {
    #[serde(with = "crate::annotations::image_bytes")]
    pub wav: Arc<Vec<u8>>,
}
impl Clip {
    pub fn from_samples(samples: &[i16]) -> Result<Self> {
        ensure!(
            !samples.is_empty() && samples.len() <= MAX_SAMPLES,
            "Recording must be between a moment and three minutes"
        );
        let size = (samples.len() * 2) as u32;
        let mut wav = Vec::with_capacity(size as usize + 44);
        wav.extend(b"RIFF");
        wav.extend((size + 36).to_le_bytes());
        wav.extend(b"WAVEfmt ");
        wav.extend(16u32.to_le_bytes());
        wav.extend(1u16.to_le_bytes());
        wav.extend(1u16.to_le_bytes());
        wav.extend(RATE.to_le_bytes());
        wav.extend((RATE * 2).to_le_bytes());
        wav.extend(2u16.to_le_bytes());
        wav.extend(16u16.to_le_bytes());
        wav.extend(b"data");
        wav.extend(size.to_le_bytes());
        for sample in samples {
            wav.extend(sample.to_le_bytes());
        }
        Ok(Self { wav: Arc::new(wav) })
    }
    pub fn validate(&self) -> Result<()> {
        if self.wav.is_empty() {
            return Ok(());
        }
        let b = self.wav.as_slice();
        ensure!(
            (46..=44 + MAX_SAMPLES * 2).contains(&b.len()) && b.len().is_multiple_of(2),
            "Invalid voice note size"
        );
        let reference = Self::from_samples(&[0])?;
        ensure!(
            b[..4] == *b"RIFF" && b[8..40] == reference.wav[8..40],
            "Unsupported voice note format"
        );
        ensure!(
            u32::from_le_bytes(b[4..8].try_into()?) as usize + 8 == b.len()
                && u32::from_le_bytes(b[40..44].try_into()?) as usize + 44 == b.len(),
            "Truncated voice note"
        );
        Ok(())
    }
    pub fn duration(&self) -> f32 {
        self.wav.len().saturating_sub(44) as f32 / (RATE * 2) as f32
    }
    pub fn samples(&self) -> Result<Vec<i16>> {
        self.validate()?;
        Ok(self
            .wav
            .get(44..)
            .unwrap_or_default()
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wave_roundtrip_and_bounds() {
        let samples = vec![-32768, -100, 0, 100, 32767];
        let clip = Clip::from_samples(&samples).unwrap();
        assert_eq!(clip.samples().unwrap(), samples);
        assert!(Clip::from_samples(&[]).is_err());
        let mut corrupt = clip.clone();
        Arc::make_mut(&mut corrupt.wav)[40] = 255;
        assert!(corrupt.validate().is_err());
        assert!(
            Clip {
                wav: Arc::new(vec![0; 44])
            }
            .validate()
            .is_err()
        );
    }
}
