use clap::{Args, Parser, Subcommand};
use commands::MoondropCommand;
use rusb::Context;
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Style};
use tabled::Table;

mod commands;
mod filter;
mod gain;
mod indicator_state;
mod usb;
mod volume_level;

#[derive(Debug, Parser)]
#[command(name = "mdrop")]
#[command(about = "A tool to control your Moondrop dongle", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// specify target device, by using the USB bus number, to which the command should be directed, ex. `03:02`
    #[arg(short = 's')]
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
}

#[derive(Debug, Args)]
struct SetArgs {
    #[command(subcommand)]
    command: SetCommands,
}

#[derive(Debug, Subcommand)]
enum SetCommands {
    /// Sets audio filter
    Filter { filter: filter::Filter },
    /// Sets gain on device to Low or High
    Gain { gain: gain::Gain },
    /// Sets current hardware volume
    Volume {
        /// Volume level between 0 and 100
        #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
        level: u8,
    },
    /// Sets indicator state to On, Off(temp), or Off
    IndicatorState {
        state: indicator_state::IndicatorState,
    },
}

fn main() {
    let args = Cli::parse();

    let mut context = match Context::new() {
        Ok(c) => c,
        Err(e) => panic!("could not initialize libusb: {}", e),
    };

    match args.command {
        Commands::Get(get) => {
            let get_cmd = get.command.unwrap_or(GetCommands::All);
            match get_cmd {
                GetCommands::All => {
                    let resp = MoondropCommand::get_any(&mut context);
                    println!("Filter: {:?}", resp.filter);
                    println!("Gain: {:?}", resp.gain);
                    println!("Indicator State: {:?}", resp.state);
                }
                GetCommands::Volume => {
                    let volume = MoondropCommand::get_volume(&mut context);
                    println!(
                        "Volume: {}%",
                        volume_level::convert_volume_to_percent(volume)
                    );
                }
            }
        }
        Commands::Set(set) => match set.command {
            SetCommands::Filter { filter } => {
                MoondropCommand::set_filter(&mut context, filter);
            }
            SetCommands::Gain { gain } => {
                MoondropCommand::set_gain(&mut context, gain);
            }
            SetCommands::Volume { level } => {
                MoondropCommand::set_volume(&mut context, level);
            }
            SetCommands::IndicatorState { state } => {
                MoondropCommand::set_indicator_state(&mut context, state);
            }
        },
        Commands::Devices => {
            let dongles = usb::detect(&mut context);
            // let dongles = vec![
            //     DeviceInfo::new(
            //         "Moondrop Dawn Pro".to_string(),
            //         "03:02".to_string(),
            //         "71%".to_string(),
            //     ),
            //     DeviceInfo::new(
            //         "Moondrop Dawn 3.5mm".to_string(),
            //         "02:01".to_string(),
            //         "40%".to_string(),
            //     ),
            //     DeviceInfo::new(
            //         "Moondrop Dawn 4.4mm".to_string(),
            //         "04:04".to_string(),
            //         "12%".to_string(),
            //     ),
            // ];
            if !dongles.is_empty() {
                let table = Table::new(dongles)
                    .with(Style::sharp())
                    .modify(Columns::last(), Alignment::right())
                    .to_string();
                println!("{table}");
            } else {
                println!("No devices present");
            }
        }
    }
}
