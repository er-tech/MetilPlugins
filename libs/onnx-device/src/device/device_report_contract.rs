use serde_json::{json, Value};

/// What a session really runs on, to put in a plugin's result so a run can be checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceReport {
    /// `cpu` or `directml:N`.
    pub device: String,
    /// The name of the GPU, when one is used.
    pub name: Option<String>,
    /// Why the GPU was not used when it was asked for (or could have been), if so.
    pub fallback: Option<String>,
}

impl DeviceReport {
    pub fn cpu(fallback: Option<String>) -> DeviceReport {
        DeviceReport {
            device: "cpu".to_string(),
            name: None,
            fallback,
        }
    }

    pub fn to_value(&self) -> Value {
        json!({ "device": self.device, "name": self.name, "fallback": self.fallback })
    }
}
