use clap::ValueEnum;

#[repr(u8)]
#[derive(Debug, Clone, Default, ValueEnum)]
pub enum IndicatorState {
    #[default]
    Enabled = 0,
    DisabledTemp = 1,
    Disabled = 2,
}
