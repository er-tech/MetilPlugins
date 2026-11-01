/// A graphics adapter, numbered as DirectML numbers them (the order of DXGI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adapter {
    pub index: u32,
    pub name: String,
    /// Memory that belongs to the GPU alone, in bytes; an integrated GPU has little or none.
    pub dedicated_bytes: u64,
    /// The software renderer Windows offers when there is no GPU.
    pub software: bool,
}

impl Adapter {
    /// Whether it is a GPU of its own: a few GiB of its own memory, as no integrated one has.
    pub fn is_discrete(&self) -> bool {
        !self.software && self.dedicated_bytes >= 2 * 1024 * 1024 * 1024
    }
}
