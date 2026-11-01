use super::adapter_contract::Adapter;

/// The graphics adapters, in the order DirectML numbers them. Empty when there is no way to list
/// them on this platform (only Windows is covered so far) or the listing fails.
#[cfg(windows)]
pub fn list_adapters() -> Vec<Adapter> {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};

    // DXGI_ADAPTER_FLAG_SOFTWARE
    const SOFTWARE: u32 = 2;
    let Ok(factory) = (unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }) else {
        return Vec::new();
    };
    let mut adapters = Vec::new();
    let mut index = 0u32;
    while let Ok(adapter) = unsafe { factory.EnumAdapters1(index) } {
        if let Ok(description) = unsafe { adapter.GetDesc1() } {
            let length = description
                .Description
                .iter()
                .position(|unit| *unit == 0)
                .unwrap_or(description.Description.len());
            adapters.push(Adapter {
                index,
                name: String::from_utf16_lossy(&description.Description[..length]),
                dedicated_bytes: description.DedicatedVideoMemory as u64,
                software: description.Flags & SOFTWARE != 0,
            });
        }
        index += 1;
    }
    adapters
}

#[cfg(not(windows))]
pub fn list_adapters() -> Vec<Adapter> {
    Vec::new()
}
