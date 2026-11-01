use super::candidate_contract::Candidate;

/// The best `count` candidates, best first, no two closer than `min_gap_ms` to each other, so the
/// picks do not all come from one scene. A candidate that scores nothing is never picked.
pub fn choose_picks(candidates: &[Candidate], count: usize, min_gap_ms: u64) -> Vec<Candidate> {
    let mut ranked: Vec<&Candidate> = candidates
        .iter()
        .filter(|candidate| candidate.score > 0.0)
        .collect();
    // Best first; at the same score the earlier picture.
    ranked.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.time_ms.cmp(&b.time_ms)));
    let mut picks: Vec<Candidate> = Vec::new();
    for candidate in ranked {
        if picks.len() == count {
            break;
        }
        let too_near = picks
            .iter()
            .any(|pick| pick.time_ms.abs_diff(candidate.time_ms) < min_gap_ms);
        if !too_near {
            picks.push(*candidate);
        }
    }
    picks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(time_ms: u64, score: f64) -> Candidate {
        Candidate {
            time_ms,
            score,
            sharpness: 0.0,
            brightness: 0.5,
        }
    }

    #[test]
    fn the_best_come_first_and_the_count_limits_them() {
        let candidates = [at(0, 0.2), at(5_000, 0.9), at(10_000, 0.5), at(15_000, 0.7)];
        let picks = choose_picks(&candidates, 2, 0);
        assert_eq!(
            picks.iter().map(|p| p.time_ms).collect::<Vec<_>>(),
            [5_000, 15_000]
        );
    }

    #[test]
    fn two_picks_are_never_closer_than_the_gap_and_the_next_best_takes_the_place() {
        let candidates = [at(0, 0.9), at(1_000, 0.8), at(2_500, 0.4), at(30_000, 0.3)];
        let picks = choose_picks(&candidates, 3, 2_000);
        assert_eq!(
            picks.iter().map(|p| p.time_ms).collect::<Vec<_>>(),
            [0, 2_500, 30_000]
        );
    }

    #[test]
    fn nothing_scoring_nothing_is_picked_and_a_tie_goes_to_the_earlier() {
        assert!(choose_picks(&[at(0, 0.0), at(1_000, 0.0)], 5, 0).is_empty());
        let tie = choose_picks(&[at(9_000, 0.5), at(3_000, 0.5)], 1, 0);
        assert_eq!(tie[0].time_ms, 3_000);
        assert!(choose_picks(&[], 3, 0).is_empty());
    }
}
