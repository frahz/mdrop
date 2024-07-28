use clap::ValueEnum;

#[repr(u8)]
#[derive(Debug, Clone, Default, ValueEnum)]
pub enum Gain {
    #[default]
    Low = 0,
    High = 1,
}
