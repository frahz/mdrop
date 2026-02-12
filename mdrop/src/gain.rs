use std::fmt::Display;

use crate::MdropError;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Gain {
    #[default]
    Low = 0,
    High = 1,
}

impl Gain {
    pub const ALL: [Gain; 2] = [Gain::Low, Gain::High];
}

impl TryFrom<u8> for Gain {
    type Error = MdropError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Gain::Low),
            1 => Ok(Gain::High),
            _ => Err(MdropError::UnknownGain(value)),
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
