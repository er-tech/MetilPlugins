use super::adapter_contract::Adapter;

/// The adapter to run on: the one with the most memory of its own. With `discrete_only` an
/// integrated GPU is not taken (it was slower than the CPU when measured), so `None` means the CPU.
/// The software renderer is never taken.
pub(crate) fn choose_adapter(adapters: &[Adapter], discrete_only: bool) -> Option<&Adapter> {
    adapters
        .iter()
        .filter(|adapter| !adapter.software)
        .filter(|adapter| !discrete_only || adapter.is_discrete())
        .max_by_key(|adapter| adapter.dedicated_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIB: u64 = 1024 * 1024 * 1024;

    fn adapter(index: u32, name: &str, gib: u64, software: bool) -> Adapter {
        Adapter {
            index,
            name: name.to_string(),
            dedicated_bytes: gib * GIB,
            software,
        }
    }

    #[test]
    fn the_adapter_with_the_most_memory_wins_whatever_its_place() {
        let adapters = [
            adapter(0, "integrated", 0, false),
            adapter(1, "discrete", 20, false),
            adapter(2, "Microsoft Basic Render Driver", 0, true),
        ];
        assert_eq!(choose_adapter(&adapters, true).unwrap().index, 1);
        assert_eq!(choose_adapter(&adapters, false).unwrap().index, 1);
    }

    #[test]
    fn an_integrated_gpu_is_taken_only_when_asked_for_and_software_never() {
        let adapters = [
            adapter(0, "integrated", 0, false),
            adapter(1, "Microsoft Basic Render Driver", 0, true),
        ];
        assert!(choose_adapter(&adapters, true).is_none());
        assert_eq!(choose_adapter(&adapters, false).unwrap().index, 0);
        assert!(choose_adapter(&[], false).is_none());
    }
}
