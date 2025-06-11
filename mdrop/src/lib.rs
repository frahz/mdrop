use std::collections::HashMap;
use std::hash::Hash;
use std::sync::mpsc;

use futures_lite::future;
use nusb::hotplug::HotplugEvent;
use nusb::transfer::{ControlIn, ControlOut};
use nusb::{DeviceId, DeviceInfo};
use tabled::Tabled;

use crate::filter::Filter;
use crate::gain::Gain;
use crate::indicator_state::IndicatorState;
use crate::volume::Volume;

pub mod filter;
pub mod gain;
pub mod indicator_state;
pub mod volume;

pub const MOONDROP_VID: u16 = 0x2fc6;
pub const DAWN_PRO_PID: u16 = 0xf06a;

const REQUEST_INDEX: u16 = 0x09A0;
const REQUEST_VALUE: u16 = 0x0000;

const REQUEST_ID_WRITE: u8 = 0xA0;
const REQUEST_ID_READ: u8 = 0xA1;

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

#[derive(Clone, Debug)]
pub struct Moondrop {
    pub devices: HashMap<DeviceId, DeviceInfo>,
    single: Option<DeviceId>,
}

impl Moondrop {
    pub fn new() -> Self {
        let devices = Self::refresh();
        let single = if devices.len() == 1 {
            devices.keys().next().cloned()
        } else {
            None
        };
        Self { devices, single }
    }

    pub fn watch(&mut self, tx: mpsc::Sender<Option<MoondropInfo>>) {
        let watch = nusb::watch_devices().unwrap();
        for event in futures_lite::stream::block_on(watch) {
            match event {
                HotplugEvent::Connected(di) => {
                    if di.vendor_id() == MOONDROP_VID {
                        self.single = Some(di.id());
                        self.devices.insert(di.id(), di);
                        tx.send(self.get_all()).expect("connect: send failed");
                        log::debug!("devices: {:?}", self.devices);
                    }
                }
                HotplugEvent::Disconnected(device_id) => {
                    log::debug!("Disconnect: {:?}", device_id);
                    if let Some(s) = self.single {
                        if device_id == s {
                            self.single = None;
                            tx.send(None).expect("disconnect: send failed");
                        }
                    }
                    self.devices.remove(&device_id);
                    log::debug!("devices: {:?}", self.devices);
                }
            }
        }
    }

    pub fn detect(&self) -> Vec<MoondropInfo> {
        self.devices
            .values()
            .map(|di| {
                let name = match di.product_string() {
                    Some(name) => name.to_string(),
                    None => match di.product_id() {
                        DAWN_PRO_PID => "MOONDROP Dawn Pro".to_string(),
                        _ => "Unknown".to_string(),
                    },
                };

                let vol_data = Self::read(di, &GET_VOLUME, 7)[VOLUME_IDX];
                let vol = Volume::from_payload(vol_data);
                let bus = format!("{:02}:{:02}", di.bus_number(), di.device_address());
                let data = Self::read(di, &GET_ANY, 7);
                MoondropInfo::new(name, bus, vol, &data)
            })
            .collect()
    }

    pub fn get_volume(&self) -> Option<Volume> {
        if let Some(id) = self.single {
            let di = self.devices.get(&id).expect("Hashmap should be populated");
            let data = Self::read(di, &GET_VOLUME, 7);
            return Some(Volume::from_payload(data[VOLUME_IDX]));
        }
        None
    }

    pub fn get_filter(&self) -> Option<Filter> {
        let all = self.get_all();
        if let Some(info) = all {
            return Some(info.filter);
        }
        None
    }

    pub fn get_gain(&self) -> Option<Gain> {
        let all = self.get_all();
        if let Some(info) = all {
            return Some(info.gain);
        }
        None
    }

    pub fn get_indicator_state(&self) -> Option<IndicatorState> {
        let all = self.get_all();
        if let Some(info) = all {
            return Some(info.indicator_state);
        }
        None
    }

    pub fn get_all(&self) -> Option<MoondropInfo> {
        if let Some(id) = self.single {
            let di = self.devices.get(&id).expect("Hashmap should be populated");
            let name = match di.product_string() {
                Some(name) => name.to_string(),
                None => match di.product_id() {
                    DAWN_PRO_PID => "MOONDROP Dawn Pro".to_string(),
                    _ => "Unknown".to_string(),
                },
            };

            let vol_data = Self::read(di, &GET_VOLUME, 7)[VOLUME_IDX];
            let vol = Volume::from_payload(vol_data);
            let bus = format!("{:02}:{:02}", di.bus_number(), di.device_address());
            let data = Self::read(di, &GET_ANY, 7);
            return Some(MoondropInfo::new(name, bus, vol, &data));
        }

        None
    }

    pub fn set_gain(&mut self, gain: Gain) {
        self.devices = Self::refresh();
        let mut cmd = Vec::from(SET_GAIN);
        cmd.push(gain as u8);
        log::debug!("Gain Command: {:?}", cmd);
        self.devices.iter().for_each(|(_, di)| {
            Self::write(di, &cmd);
        });
    }

    pub fn set_volume(&mut self, level: Volume) {
        self.devices = Self::refresh();
        let value = level.to_payload();
        log::debug!("Volume Level: {level} clamped: {value}");
        let mut cmd = Vec::from(SET_VOLUME);
        // FIXME: might be incorrect
        cmd.push(value);
        log::debug!("Volume Command: {:?}", cmd);
        self.devices.iter().for_each(|(_, di)| {
            Self::write(di, &cmd);
        });
    }

    pub fn set_filter(&mut self, filter: Filter) {
        self.devices = Self::refresh();
        let mut cmd = Vec::from(SET_FILTER);
        cmd.push(filter as u8);
        log::debug!("Filter Command: {:?}", cmd);
        self.devices.iter().for_each(|(_, di)| {
            Self::write(di, &cmd);
        });
    }

    pub fn set_indicator_state(&mut self, indicator_state: IndicatorState) {
        self.devices = Self::refresh();
        let mut cmd = Vec::from(SET_INDICATOR_STATE);
        cmd.push(indicator_state as u8);
        log::debug!("IndicatorState Command: {:?}", cmd);
        self.devices.iter().for_each(|(_, di)| {
            Self::write(di, &cmd);
        });
    }

    fn read(di: &DeviceInfo, cmd: &[u8], length: u16) -> Vec<u8> {
        let device = di.open().expect("device open failed");
        let _ = future::block_on(device.control_out(ControlOut {
            control_type: nusb::transfer::ControlType::Vendor,
            recipient: nusb::transfer::Recipient::Other,
            request: REQUEST_ID_WRITE,
            value: REQUEST_VALUE,
            index: REQUEST_INDEX,
            data: cmd,
        }))
        .into_result()
        .expect("write failed");
        future::block_on(device.control_in(ControlIn {
            control_type: nusb::transfer::ControlType::Vendor,
            recipient: nusb::transfer::Recipient::Other,
            request: REQUEST_ID_READ,
            value: REQUEST_VALUE,
            index: REQUEST_INDEX,
            length,
        }))
        .into_result()
        .expect("read failed")
    }

    fn write(di: &DeviceInfo, cmd: &[u8]) {
        let device = di.open().expect("device open failed");
        let _ = future::block_on(device.control_out(ControlOut {
            control_type: nusb::transfer::ControlType::Vendor,
            recipient: nusb::transfer::Recipient::Other,
            request: REQUEST_ID_WRITE,
            value: REQUEST_VALUE,
            index: REQUEST_INDEX,
            data: cmd,
        }))
        .into_result()
        .expect("write failed");
    }

    fn refresh() -> HashMap<DeviceId, DeviceInfo> {
        nusb::list_devices()
            .unwrap()
            .filter(|d| d.vendor_id() == MOONDROP_VID)
            .map(|d| (d.id(), d))
            .collect()
    }
}

impl Default for Moondrop {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for Moondrop {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.single.hash(state);
    }
}

#[derive(Clone, Debug, Tabled)]
#[tabled(rename_all = "snake")]
pub struct MoondropInfo {
    pub name: String,
    pub bus: String,
    pub volume: Volume,
    pub filter: Filter,
    pub gain: Gain,
    pub indicator_state: IndicatorState,
}

impl MoondropInfo {
    pub fn new(name: String, bus: String, volume: Volume, data: &[u8]) -> Self {
        let filter = Filter::from(data[FILTER_IDX]);
        let gain = Gain::from(data[GAIN_IDX]);
        let state = IndicatorState::from(data[INDICATOR_STATE_IDX]);
        Self {
            name,
            bus,
            volume,
            filter,
            gain,
            indicator_state: state,
        }
    }
}
