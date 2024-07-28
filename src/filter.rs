use clap::ValueEnum;

#[repr(u8)]
#[derive(Debug, Clone, Default, ValueEnum)]
pub enum Filter {
    #[default]
    FastRollOffLowLatency = 0,
    FastRollOffPhaseCompensated = 1,
    SlowRollOffLowLatency = 2,
    SlowRollOffPhaseCompensated = 3,
    NonOversampling = 4,
}
