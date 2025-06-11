use clap::{Args, Parser, Subcommand};
use mdrop::Moondrop;
use mdrop::filter::Filter;
use mdrop::gain::Gain;
use mdrop::indicator_state::IndicatorState;
use mdrop::volume::Volume;
use tabled::Table;
use tabled::settings::themes::ColumnNames;
use tabled::settings::{Alignment, Style};

#[derive(Debug, Parser)]
#[command(name = "mdrop")]
#[command(about = "A tool to control your Moondrop dongle", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// specify target device, by using the USB bus number, to which the command should be directed, ex. `03:02`
    #[arg(short = 's', global = true)]
    device: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Gets status of Moondrop dongle
    Get(GetArgs),
    /// Sets various values in your Moondrop dongle
    Set(SetArgs),
    /// Lists all the Moondrop dongles connected to the PC
    Devices,
}

#[derive(Debug, Args)]
struct GetArgs {
    #[command(subcommand)]
    command: Option<GetCommands>,
}

#[derive(Debug, Subcommand)]
enum GetCommands {
    /// Gets status for filter, gain, and indicator state
    All,
    /// Gets current hardware volume of Moondrop dongle
    Volume,
    /// Gets audio filter
    Filter,
    /// Gets gain on device to Low or High
    Gain,
    /// Gets indicator state to On, Off(temp), or Off
    IndicatorState,
}

#[derive(Debug, Args)]
struct SetArgs {
    #[command(subcommand)]
    command: SetCommands,
}

#[derive(Debug, Subcommand)]
enum SetCommands {
    /// Sets audio filter
    Filter { filter: Filter },
    /// Sets gain on device to Low or High
    Gain { gain: Gain },
    /// Sets current hardware volume
    Volume {
        /// Volume level between 0 and 100
        #[arg(value_parser = clap::value_parser!(u32).range(0..=100))]
        level: u32,
    },
    /// Sets indicator state to On, Off(temp), or Off
    IndicatorState { state: IndicatorState },
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    log::debug!("Device: {:?}", args.device);

    let mut moondrop = Moondrop::new();

    match args.command {
        Commands::Get(get) => {
            let get_cmd = get.command.unwrap_or(GetCommands::All);
            match get_cmd {
                GetCommands::All => {
                    if let Some(dongle) = moondrop.get_all() {
                        let table = Table::new([dongle])
                            .with(Style::sharp().remove_horizontals())
                            .with(ColumnNames::default().alignment(Alignment::center()))
                            .to_string();
                        println!("{table}");
                    } else {
                        println!("No Moondrop dongle connected.");
                    }
                }
                GetCommands::Volume => {
                    if let Some(volume) = moondrop.get_volume() {
                        println!("Volume: {}", volume);
                    } else {
                        println!("No Moondrop dongle connected.");
                    }
                }
                GetCommands::Filter => {
                    if let Some(filter) = moondrop.get_filter() {
                        println!("Filter: {filter}")
                    } else {
                        println!("No Moondrop dongle connected.");
                    }
                }
                GetCommands::Gain => {
                    if let Some(gain) = moondrop.get_gain() {
                        println!("Gain: {gain}");
                    } else {
                        println!("No Moondrop dongle connected.");
                    }
                }
                GetCommands::IndicatorState => {
                    if let Some(state) = moondrop.get_indicator_state() {
                        println!("Indicator State: {state}");
                    } else {
                        println!("No Moondrop dongle connected.");
                    }
                }
            }
        }
        Commands::Set(set) => match set.command {
            SetCommands::Filter { filter } => moondrop.set_filter(filter),
            SetCommands::Gain { gain } => moondrop.set_gain(gain),
            SetCommands::Volume { level } => moondrop.set_volume(Volume::new(level)),
            SetCommands::IndicatorState { state } => moondrop.set_indicator_state(state),
        },
        Commands::Devices => {
            let dongles = moondrop.detect();
            if !dongles.is_empty() {
                let table = Table::new(dongles)
                    .with(Style::sharp().remove_horizontals())
                    .with(ColumnNames::default().alignment(Alignment::center()))
                    .to_string();
                println!("{table}");
            } else {
                println!("No devices present");
            }
        }
    }
}
