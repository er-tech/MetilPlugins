use super::volume_window_contract::VolumeWindow;

/// A stretch of the sound that stayed below a level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Silence {
    pub start_ms: u64,
    pub end_ms: u64,
}

/// The stretches where the sound stays below `threshold_db` for at least `min_ms`: runs of windows
/// that are all quieter than the threshold, joined. A stretch shorter than `min_ms` is ignored.
pub fn find_silences(windows: &[VolumeWindow], threshold_db: f64, min_ms: u64) -> Vec<Silence> {
    let mut silences = Vec::new();
    let mut current: Option<Silence> = None;
    for window in windows {
        if window.rms_db < threshold_db {
            match &mut current {
                Some(silence) => silence.end_ms = window.end_ms,
                None => {
                    current = Some(Silence {
                        start_ms: window.start_ms,
                        end_ms: window.end_ms,
                    });
                }
            }
        } else if let Some(silence) = current.take() {
            silences.push(silence);
        }
    }
    silences.extend(current);
    silences
        .into_iter()
        .filter(|silence| silence.end_ms - silence.start_ms >= min_ms)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One window of 100 ms for each level.
    fn windows(levels: &[f64]) -> Vec<VolumeWindow> {
        levels
            .iter()
            .enumerate()
            .map(|(index, db)| VolumeWindow {
                start_ms: index as u64 * 100,
                end_ms: (index as u64 + 1) * 100,
                rms_db: *db,
                peak_db: *db,
                bands: None,
            })
            .collect()
    }

    #[test]
    fn joins_quiet_windows_and_drops_the_stretches_that_are_too_short() {
        let levels = [
            -10.0, -80.0, -80.0, -80.0, -10.0, -80.0, -10.0, -90.0, -90.0,
        ];
        let found = find_silences(&windows(&levels), -50.0, 250);
        assert_eq!(
            found,
            [Silence {
                start_ms: 100,
                end_ms: 400
            }]
        );
        // A shorter minimum finds the single quiet window and the one at the end too.
        let all = find_silences(&windows(&levels), -50.0, 100);
        assert_eq!(all.len(), 3);
        assert_eq!(
            all[2],
            Silence {
                start_ms: 700,
                end_ms: 900
            }
        );
    }

    #[test]
    fn a_threshold_decides_what_is_quiet_and_nothing_quiet_gives_nothing() {
        let levels = [-30.0, -30.0, -30.0];
        assert!(find_silences(&windows(&levels), -50.0, 0).is_empty());
        assert_eq!(find_silences(&windows(&levels), -20.0, 0).len(), 1);
        assert!(find_silences(&[], -50.0, 0).is_empty());
    }
}
