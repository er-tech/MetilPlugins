use std::sync::atomic::{AtomicU64, Ordering};

/// A name no other file of this program has: the prefix, the process and a counter. Two programs
/// share a work folder, and a program writes many files in a run.
pub fn unique_work_file_name(prefix: &str, extension: &str) -> String {
    static NUMBER: AtomicU64 = AtomicU64::new(0);
    format!(
        "{prefix}-{}-{}.{extension}",
        std::process::id(),
        NUMBER.fetch_add(1, Ordering::Relaxed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_names_are_never_the_same_and_keep_the_prefix_and_the_extension() {
        let (a, b) = (
            unique_work_file_name("shot", "png"),
            unique_work_file_name("shot", "png"),
        );
        assert_ne!(a, b);
        assert!(a.starts_with("shot-") && a.ends_with(".png"));
    }
}
