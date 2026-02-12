use std::fmt::Display;

use crate::MdropError;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum IndicatorState {
    #[default]
    Enabled = 0,
    DisabledTemp = 1,
    Disabled = 2,
}

impl IndicatorState {
    pub const ALL: [IndicatorState; 3] = [
        IndicatorState::Enabled,
        IndicatorState::DisabledTemp,
        IndicatorState::Disabled,
    ];
}

impl TryFrom<u8> for IndicatorState {
    type Error = MdropError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(IndicatorState::Enabled),
            1 => Ok(IndicatorState::DisabledTemp),
            2 => Ok(IndicatorState::Disabled),
            _ => Err(MdropError::UnknownIndicatorState(value)),
        }
    }
}

impl Display for IndicatorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndicatorState::Enabled => write!(f, "Enabled"),
            IndicatorState::DisabledTemp => write!(f, "Temporarily Disabled"),
            IndicatorState::Disabled => write!(f, "Disabled"),
        }
    }
}
