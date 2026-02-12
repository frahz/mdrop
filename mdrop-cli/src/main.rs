use clap::{Args, Parser, Subcommand};
use mdrop::filter::Filter;
use mdrop::gain::Gain;
use mdrop::indicator_state::IndicatorState;
use mdrop::volume::Volume;
use mdrop::DeviceSelector;
use mdrop::Moondrop;
use mdrop::MoondropInfo;
use serde::Serialize;
use tabled::settings::themes::ColumnNames;
use tabled::settings::{Alignment, Style};
use tabled::Table;
use tabled::Tabled;

#[derive(Debug, Parser)]
#[command(name = "mdrop")]
#[command(about = "A tool to control your Moondrop dongle", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// specify target device, by using the USB bus number, to which the command should be directed, ex. `03:02`
    #[arg(short = 's', global = true)]
    device: Option<String>,

    /// print output as JSON
    #[arg(long, global = true)]
    json: bool,
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

#[derive(Clone, Serialize, Tabled)]
struct DongleOutput {
    name: String,
    bus: String,
    volume: u32,
    filter: String,
    gain: String,
    indicator_state: String,
}

impl From<MoondropInfo> for DongleOutput {
    fn from(value: MoondropInfo) -> Self {
        Self {
            name: value.name,
            bus: value.bus,
            volume: value.volume.inner(),
            filter: value.filter.to_string(),
            gain: value.gain.to_string(),
            indicator_state: value.indicator_state.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ValueOutput<T: Serialize> {
    value: T,
}

fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    let json = serde_json::to_string(value).map_err(|err| err.to_string())?;
    println!("{json}");
    Ok(())
}

fn main() {
    env_logger::init();

    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = Cli::parse();
    let selector = DeviceSelector::from_bus(args.device.as_deref());
    let mut moondrop = Moondrop::new();

    match args.command {
        Commands::Get(get) => {
            let get_cmd = get.command.unwrap_or(GetCommands::All);
            match get_cmd {
                GetCommands::All => {
                    let dongle = moondrop
                        .get_all(selector.clone())
                        .map_err(|err| err.to_string())?;
                    if args.json {
                        print_json(&DongleOutput::from(dongle))?;
                    } else {
                        let table = Table::new([DongleOutput::from(dongle)])
                            .with(Style::sharp().remove_horizontals())
                            .with(ColumnNames::head().alignment(Alignment::center()))
                            .to_string();
                        println!("{table}");
                    }
                }
                GetCommands::Volume => {
                    let volume = moondrop
                        .get_volume(selector.clone())
                        .map_err(|err| err.to_string())?;
                    if args.json {
                        print_json(&ValueOutput {
                            value: volume.inner(),
                        })?;
                    } else {
                        println!("Volume: {}", volume);
                    }
                }
                GetCommands::Filter => {
                    let filter = moondrop
                        .get_filter(selector.clone())
                        .map_err(|err| err.to_string())?;
                    if args.json {
                        print_json(&ValueOutput {
                            value: filter.to_string(),
                        })?;
                    } else {
                        println!("Filter: {filter}")
                    }
                }
                GetCommands::Gain => {
                    let gain = moondrop
                        .get_gain(selector.clone())
                        .map_err(|err| err.to_string())?;
                    if args.json {
                        print_json(&ValueOutput {
                            value: gain.to_string(),
                        })?;
                    } else {
                        println!("Gain: {gain}");
                    }
                }
                GetCommands::IndicatorState => {
                    let state = moondrop
                        .get_indicator_state(selector.clone())
                        .map_err(|err| err.to_string())?;
                    if args.json {
                        print_json(&ValueOutput {
                            value: state.to_string(),
                        })?;
                    } else {
                        println!("Indicator State: {state}");
                    }
                }
            }
        }
        Commands::Set(set) => match set.command {
            SetCommands::Filter { filter } => moondrop
                .set_filter(selector.clone(), filter)
                .map_err(|err| err.to_string())?,
            SetCommands::Gain { gain } => moondrop
                .set_gain(selector.clone(), gain)
                .map_err(|err| err.to_string())?,
            SetCommands::Volume { level } => moondrop
                .set_volume(selector.clone(), Volume::new(level))
                .map_err(|err| err.to_string())?,
            SetCommands::IndicatorState { state } => moondrop
                .set_indicator_state(selector.clone(), state)
                .map_err(|err| err.to_string())?,
        },
        Commands::Devices => {
            let dongles = moondrop.detect().map_err(|err| err.to_string())?;
            if args.json {
                let output: Vec<DongleOutput> =
                    dongles.into_iter().map(DongleOutput::from).collect();
                print_json(&output)?;
            } else if !dongles.is_empty() {
                let output: Vec<DongleOutput> =
                    dongles.into_iter().map(DongleOutput::from).collect();
                let table = Table::new(output)
                    .with(Style::sharp().remove_horizontals())
                    .with(ColumnNames::head().alignment(Alignment::center()))
                    .to_string();
                println!("{table}");
            } else {
                println!("No devices present");
            }
        }
    }

    Ok(())
}
