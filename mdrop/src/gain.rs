use std::fmt::Display;

use clap::ValueEnum;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Gain {
    #[default]
    Low = 0,
    High = 1,
}

impl Gain {
    pub const ALL: [Gain; 2] = [Gain::Low, Gain::High];
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

impl Display for Gain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gain::Low => write!(f, "Low"),
            Gain::High => write!(f, "High"),
        }
    }
}
