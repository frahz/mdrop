use clap::{Args, Parser, Subcommand};
use rusb::Context;

mod filter;
mod gain;
mod indicator_state;
mod usb;
mod volume_level;

const GET_ANY: [u8; 3] = [0xC0, 0xA5, 0xA3];
const GET_VOLUME: [u8; 3] = [0xC0, 0xA5, 0xA2];
const SET_FILTER: [u8; 3] = [0xC0, 0xA5, 0x01];
const SET_GAIN: [u8; 3] = [0xC0, 0xA5, 0x02];
const SET_VOLUME: [u8; 3] = [0xC0, 0xA5, 0x04];
const SET_INDICATOR_STATE: [u8; 3] = [0xC0, 0xA5, 0x06];

#[derive(Debug, Parser)]
#[command(name = "mdrop")]
#[command(about = "A tool to control your Moondrop dongle", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// specify target device to which the command should be directed
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
    Volume { level: u8 },
    /// Sets indicator state to On, Off(temp), or Off
    IndicatorState {
        state: indicator_state::IndicatorState,
    },
}

fn main() {
    // println!(
    //     "Moondrop Dawn Pro Device: VID={:#04x} PID={:#04x}",
    //     DAWN_PRO_VID, DAWN_PRO_PID
    // );

    let args = Cli::parse();

    let mut context = match Context::new() {
        Ok(c) => c,
        Err(e) => panic!("could not initialize libusb: {}", e),
    };

    match args.command {
        Commands::Get(get) => {
            let get_cmd = get.command.unwrap_or(GetCommands::All);
            let mut data = [0u8; 7];
            match get_cmd {
                GetCommands::All => {
                    usb::get(&mut context, &GET_ANY, &mut data);
                    println!("Filter: {}", data[3]);
                    println!("Gain: {}", data[4]);
                    println!("Indicator State: {}", data[5]);
                }
                GetCommands::Volume => {
                    usb::get(&mut context, &GET_VOLUME, &mut data);
                    println!("Volume: {}%", volume_level::convert_volume(data[4]));
                }
            }
        }
        Commands::Set(set) => match set.command {
            SetCommands::Filter { filter } => {
                println!("Filter: {:?}", filter);
                let mut cmd = Vec::from(SET_FILTER);
                cmd.push(filter as u8);
                println!("Filter Command: {:?}", cmd);
                usb::set(&mut context, &cmd);
            }
            SetCommands::Gain { gain } => {
                println!("New Gain: {:?}", gain);
                let mut cmd = Vec::from(SET_GAIN);
                cmd.push(gain as u8);
                println!("Gain Command: {:?}", cmd);
                usb::set(&mut context, &cmd);
            }
            SetCommands::Volume { level } => {
                println!("Volume Level: {level}");
                let mut cmd = Vec::from(SET_VOLUME);
                // FIXME: might be incorrect
                cmd.push(level);
                println!("Volume Command: {:?}", cmd);
            }
            SetCommands::IndicatorState { state } => {
                println!("New IndicatorState: {:?}", state);
                let mut cmd = Vec::from(SET_INDICATOR_STATE);
                cmd.push(state as u8);
                println!("IndicatorState Command: {:?}", cmd);
                usb::set(&mut context, &cmd);
            }
        },
        Commands::Devices => {
            let dongles = usb::detect(&mut context);
            for dongle in dongles {
                println!("Name: {}", dongle);
            }
        }
    }
}
