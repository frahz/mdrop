use clap::ValueEnum;

#[repr(u8)]
#[derive(Debug, Clone, Default, ValueEnum)]
pub enum Gain {
    #[default]
    Low = 0,
    High = 1,
}

impl From<u8> for Gain {
    fn from(value: u8) -> Self {
        match value {
            0 => Gain::Low,
            1 => Gain::High,
            _ => Gain::Low,
        }
    }
}
