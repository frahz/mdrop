use clap::ValueEnum;

#[repr(u8)]
#[derive(Debug, Clone, Default, ValueEnum)]
pub enum IndicatorState {
    #[default]
    Enabled = 0,
    DisabledTemp = 1,
    Disabled = 2,
}

impl From<u8> for IndicatorState {
    fn from(value: u8) -> Self {
        match value {
            0 => IndicatorState::Enabled,
            1 => IndicatorState::DisabledTemp,
            2 => IndicatorState::Disabled,
            _ => IndicatorState::Enabled,
        }
    }
}
