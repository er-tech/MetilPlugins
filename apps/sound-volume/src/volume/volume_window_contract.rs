use serde_json::{json, Value};

use super::band_levels_algorithm::BandLevels;

/// How loud one stretch of the sound is.
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeWindow {
    pub start_ms: u64,
    pub end_ms: u64,
    /// The average level, how loud it sounds, in dB relative to full scale (0 is the loudest).
    pub rms_db: f64,
    /// The loudest single instant, in dB.
    pub peak_db: f64,
    /// The level of the low, mid and high bands, when they were asked for.
    pub bands: Option<BandLevels>,
}

impl VolumeWindow {
    /// What the action gives for this window; levels are rounded to a tenth of a decibel.
    pub fn to_value(&self) -> Value {
        let mut value = json!({
            "startMs": self.start_ms,
            "endMs": self.end_ms,
            "rmsDb": round_db(self.rms_db),
            "value": round_db(self.rms_db),
            "peakDb": round_db(self.peak_db),
        });
        if let Some(bands) = self.bands {
            value["bands"] = json!({
                "low": round_db(bands.low),
                "mid": round_db(bands.mid),
                "high": round_db(bands.high),
            });
        }
        value
    }
}

/// A level rounded to a tenth of a decibel: more is noise in a measurement like this.
pub fn round_db(db: f64) -> f64 {
    (db * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_reads_as_times_and_levels_rounded_to_a_tenth_of_a_decibel() {
        let window = VolumeWindow {
            start_ms: 1_000,
            end_ms: 2_000,
            rms_db: -12.3456,
            peak_db: -3.04,
            bands: Some(BandLevels {
                low: -20.04,
                mid: -13.0,
                high: -100.0,
            }),
        };
        assert_eq!(
            window.to_value(),
            json!({ "startMs": 1000, "endMs": 2000, "rmsDb": -12.3, "value": -12.3, "peakDb": -3.0,
                    "bands": { "low": -20.0, "mid": -13.0, "high": -100.0 } })
        );
        let plain = VolumeWindow {
            bands: None,
            ..window
        };
        assert!(plain.to_value().get("bands").is_none());
    }
}
