use rusb::UsbContext;

use crate::filter::Filter;
use crate::gain::Gain;
use crate::indicator_state::IndicatorState;
use crate::{usb, volume_level};

const GET_ANY: [u8; 3] = [0xC0, 0xA5, 0xA3];
const GET_VOLUME: [u8; 3] = [0xC0, 0xA5, 0xA2];
const SET_FILTER: [u8; 3] = [0xC0, 0xA5, 0x01];
const SET_GAIN: [u8; 3] = [0xC0, 0xA5, 0x02];
const SET_VOLUME: [u8; 3] = [0xC0, 0xA5, 0x04];
const SET_INDICATOR_STATE: [u8; 3] = [0xC0, 0xA5, 0x06];

const VOLUME_IDX: usize = 4;
const FILTER_IDX: usize = 3;
const GAIN_IDX: usize = 4;
const INDICATOR_STATE_IDX: usize = 5;

pub struct MoondropCommand;

pub struct MoondropResponse {
    pub filter: Filter,
    pub gain: Gain,
    pub state: IndicatorState,
}

impl MoondropResponse {
    pub fn new(data: &[u8]) -> Self {
        let filter = Filter::from(data[FILTER_IDX]);
        let gain = Gain::from(data[GAIN_IDX]);
        let state = IndicatorState::from(data[INDICATOR_STATE_IDX]);
        Self {
            filter,
            gain,
            state,
        }
    }
}

impl MoondropCommand {
    pub fn get_any<T: UsbContext>(context: &mut T) -> MoondropResponse {
        let mut data = [0u8; 7];
        usb::get(context, &GET_ANY, &mut data);
        MoondropResponse::new(&data)
    }

    pub fn get_volume<T: UsbContext>(context: &mut T) -> u8 {
        let mut data = [0u8; 7];
        usb::get(context, &GET_VOLUME, &mut data);
        data[VOLUME_IDX]
    }

    pub fn set_gain<T: UsbContext>(context: &mut T, gain: Gain) {
        println!("New Gain: {:?}", gain);
        let mut cmd = Vec::from(SET_GAIN);
        cmd.push(gain as u8);
        println!("Gain Command: {:?}", cmd);
        usb::set(context, &cmd);
    }

    pub fn set_volume<T: UsbContext>(context: &mut T, level: u8) {
        let value = volume_level::convert_volume_to_payload(level);
        println!("Volume Level: {level} clamped: {value}");
        let mut cmd = Vec::from(SET_VOLUME);
        // FIXME: might be incorrect
        cmd.push(value);
        println!("Volume Command: {:?}", cmd);
        usb::set(context, &cmd);
    }

    pub fn set_filter<T: UsbContext>(context: &mut T, filter: Filter) {
        println!("Filter: {:?}", filter);
        let mut cmd = Vec::from(SET_FILTER);
        cmd.push(filter as u8);
        println!("Filter Command: {:?}", cmd);
        usb::set(context, &cmd);
    }

    pub fn set_indicator_state<T: UsbContext>(context: &mut T, indicator_state: IndicatorState) {
        println!("New IndicatorState: {:?}", indicator_state);
        let mut cmd = Vec::from(SET_INDICATOR_STATE);
        cmd.push(indicator_state as u8);
        println!("IndicatorState Command: {:?}", cmd);
        usb::set(context, &cmd);
    }
}
