const DEFAULT_CHANNELS: u16 = 2;

#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// None means auto-detect from device.
    pub sample_rate: Option<u32>,
    pub channels: u16,
    pub device_name: Option<String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            sample_rate: None,
            channels: DEFAULT_CHANNELS,
            device_name: None,
        }
    }
}

impl EngineConfig {
    pub fn sample_rate_or(&self, fallback: u32) -> u32 {
        self.sample_rate.unwrap_or(fallback)
    }

    pub fn with_sample_rate(mut self, sr: u32) -> Self {
        self.sample_rate = Some(sr);
        self
    }

    pub fn with_device(mut self, name: impl Into<String>) -> Self {
        self.device_name = Some(name.into());
        self
    }
}
