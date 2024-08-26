use clap::ValueEnum;

#[repr(u8)]
#[derive(Debug, Clone, Default, ValueEnum)]
pub enum Filter {
    #[default]
    #[clap(alias = "froll")]
    FastRollOffLowLatency = 0,
    #[clap(alias = "fropc")]
    FastRollOffPhaseCompensated = 1,
    #[clap(alias = "sroll")]
    SlowRollOffLowLatency = 2,
    #[clap(alias = "sropc")]
    SlowRollOffPhaseCompensated = 3,
    #[clap(alias = "no")]
    NonOversampling = 4,
}

impl From<u8> for Filter {
    fn from(value: u8) -> Self {
        match value {
            0 => Filter::FastRollOffLowLatency,
            1 => Filter::FastRollOffPhaseCompensated,
            2 => Filter::SlowRollOffLowLatency,
            3 => Filter::SlowRollOffPhaseCompensated,
            4 => Filter::NonOversampling,
            _ => Filter::FastRollOffLowLatency,
        }
    }
}
