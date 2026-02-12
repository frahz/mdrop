use std::fmt::Display;

use crate::MdropError;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Filter {
    #[default]
    #[cfg_attr(feature = "clap", clap(alias = "froll"))]
    FastRollOffLowLatency = 0,
    #[cfg_attr(feature = "clap", clap(alias = "fropc"))]
    FastRollOffPhaseCompensated = 1,
    #[cfg_attr(feature = "clap", clap(alias = "sroll"))]
    SlowRollOffLowLatency = 2,
    #[cfg_attr(feature = "clap", clap(alias = "sropc"))]
    SlowRollOffPhaseCompensated = 3,
    #[cfg_attr(feature = "clap", clap(alias = "no"))]
    NonOversampling = 4,
}

impl Filter {
    pub const ALL: [Filter; 5] = [
        Filter::FastRollOffLowLatency,
        Filter::FastRollOffPhaseCompensated,
        Filter::SlowRollOffLowLatency,
        Filter::SlowRollOffPhaseCompensated,
        Filter::NonOversampling,
    ];
}

impl TryFrom<u8> for Filter {
    type Error = MdropError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Filter::FastRollOffLowLatency),
            1 => Ok(Filter::FastRollOffPhaseCompensated),
            2 => Ok(Filter::SlowRollOffLowLatency),
            3 => Ok(Filter::SlowRollOffPhaseCompensated),
            4 => Ok(Filter::NonOversampling),
            _ => Err(MdropError::UnknownFilter(value)),
        }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::FastRollOffLowLatency => write!(f, "Fast roll-off, low-latency"),
            Filter::FastRollOffPhaseCompensated => write!(f, "Fast roll-off, phase-compensated"),
            Filter::SlowRollOffLowLatency => write!(f, "Slow roll-off, low-latency"),
            Filter::SlowRollOffPhaseCompensated => write!(f, "Slow roll-off, phase-compensated"),
            Filter::NonOversampling => write!(f, "Non-oversampling"),
        }
    }
}
