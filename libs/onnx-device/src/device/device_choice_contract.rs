/// Where the user wants a model to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DeviceChoice {
    /// The GPU when there is a discrete one, the CPU otherwise.
    #[default]
    Auto,
    Cpu,
    /// The best GPU, even an integrated one (`None`), or the adapter with this number.
    Gpu(Option<u32>),
}

impl DeviceChoice {
    /// What to run on when the caller asked for `asked` and the host grants `granted` (the word it
    /// offered in `hello`, see `Cancel::granted_device`): a caller that left it to `auto` takes what
    /// the host grants, and a caller that named a device keeps it. A grant that is not a device
    /// word is ignored.
    pub fn resolve(asked: DeviceChoice, granted: Option<&str>) -> DeviceChoice {
        match (asked, granted.and_then(DeviceChoice::parse)) {
            (DeviceChoice::Auto, Some(grant)) => grant,
            _ => asked,
        }
    }

    /// Reads `auto`, `cpu`, `gpu` or `gpu:N` (N from 0), in any case. `None` when it is anything
    /// else, so a plugin can answer with a 400.
    pub fn parse(text: &str) -> Option<DeviceChoice> {
        let text = text.trim().to_lowercase();
        match text.as_str() {
            "auto" => Some(DeviceChoice::Auto),
            "cpu" => Some(DeviceChoice::Cpu),
            "gpu" => Some(DeviceChoice::Gpu(None)),
            other => other
                .strip_prefix("gpu:")
                .and_then(|number| number.parse().ok())
                .map(|number| DeviceChoice::Gpu(Some(number))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_words_a_user_may_give_are_read_and_anything_else_is_refused() {
        assert_eq!(DeviceChoice::parse("auto"), Some(DeviceChoice::Auto));
        assert_eq!(DeviceChoice::parse(" CPU "), Some(DeviceChoice::Cpu));
        assert_eq!(DeviceChoice::parse("gpu"), Some(DeviceChoice::Gpu(None)));
        assert_eq!(
            DeviceChoice::parse("GPU:1"),
            Some(DeviceChoice::Gpu(Some(1)))
        );
        assert_eq!(DeviceChoice::parse("gpu:x"), None);
        assert_eq!(DeviceChoice::parse("cuda"), None);
    }

    #[test]
    fn auto_takes_the_grant_and_a_named_device_keeps_itself() {
        let cpu = DeviceChoice::Cpu;
        assert_eq!(DeviceChoice::resolve(DeviceChoice::Auto, Some("cpu")), cpu);
        assert_eq!(
            DeviceChoice::resolve(DeviceChoice::Auto, Some("gpu:1")),
            DeviceChoice::Gpu(Some(1))
        );
        assert_eq!(
            DeviceChoice::resolve(DeviceChoice::Auto, None),
            DeviceChoice::Auto
        );
        assert_eq!(
            DeviceChoice::resolve(DeviceChoice::Auto, Some("x")),
            DeviceChoice::Auto
        );
        assert_eq!(DeviceChoice::resolve(cpu, Some("gpu:0")), cpu);
    }
}
