use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

use nusb::hotplug::HotplugEvent;
use nusb::transfer::{ControlIn, ControlOut};
use nusb::{DeviceId, DeviceInfo, MaybeFuture};

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
const CONTROL_TIMEOUT: Duration = Duration::from_millis(500);

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
const ALL_PAYLOAD_LEN: usize = 7;

pub type MdropResult<T> = Result<T, MdropError>;

#[derive(Debug)]
pub enum MdropError {
    NoDevice,
    AmbiguousDeviceSelection(usize),
    InvalidBusFormat(String),
    DeviceNotFound(String),
    DeviceWatchFailed(String),
    UsbOpenFailed(String),
    UsbWriteFailed(String),
    UsbReadFailed(String),
    DeviceEnumerationFailed(String),
    ChannelSendFailed(String),
    InvalidPayloadLength { expected: usize, got: usize },
    UnknownFilter(u8),
    UnknownGain(u8),
    UnknownIndicatorState(u8),
}

impl std::fmt::Display for MdropError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MdropError::NoDevice => write!(f, "No Moondrop dongle connected"),
            MdropError::AmbiguousDeviceSelection(count) => write!(
                f,
                "{count} Moondrop dongles detected; specify one with --device BB:DD"
            ),
            MdropError::InvalidBusFormat(bus) => {
                write!(f, "Invalid bus selector '{bus}', expected format BB:DD")
            }
            MdropError::DeviceNotFound(bus) => write!(f, "No Moondrop dongle found for bus {bus}"),
            MdropError::DeviceWatchFailed(err) => write!(f, "Failed to watch USB devices: {err}"),
            MdropError::UsbOpenFailed(err) => write!(f, "Failed to open USB device: {err}"),
            MdropError::UsbWriteFailed(err) => write!(f, "USB write failed: {err}"),
            MdropError::UsbReadFailed(err) => write!(f, "USB read failed: {err}"),
            MdropError::DeviceEnumerationFailed(err) => {
                write!(f, "Failed to enumerate USB devices: {err}")
            }
            MdropError::ChannelSendFailed(err) => write!(f, "Failed to send update: {err}"),
            MdropError::InvalidPayloadLength { expected, got } => {
                write!(f, "Invalid payload length: expected {expected}, got {got}")
            }
            MdropError::UnknownFilter(value) => write!(f, "Unknown filter payload value: {value}"),
            MdropError::UnknownGain(value) => write!(f, "Unknown gain payload value: {value}"),
            MdropError::UnknownIndicatorState(value) => {
                write!(f, "Unknown indicator state payload value: {value}")
            }
        }
    }
}

impl std::error::Error for MdropError {}

#[derive(Clone, Debug, Default)]
pub enum DeviceSelector {
    #[default]
    Auto,
    Bus(String),
}

impl DeviceSelector {
    pub fn from_bus(bus: Option<&str>) -> Self {
        match bus {
            Some(value) => Self::Bus(value.to_string()),
            None => Self::Auto,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Moondrop {
    pub devices: HashMap<DeviceId, DeviceInfo>,
}

impl Moondrop {
    pub fn new() -> Self {
        let devices = Self::refresh().unwrap_or_default();
        Self { devices }
    }

    pub fn watch(&mut self, tx: mpsc::Sender<Option<MoondropInfo>>) -> MdropResult<()> {
        let watch =
            nusb::watch_devices().map_err(|err| MdropError::DeviceWatchFailed(err.to_string()))?;
        for event in futures_lite::stream::block_on(watch) {
            match event {
                HotplugEvent::Connected(di) => {
                    if di.vendor_id() == MOONDROP_VID {
                        self.devices.insert(di.id(), di);
                        let current = self.get_all(DeviceSelector::Auto).ok();
                        tx.send(current)
                            .map_err(|err| MdropError::ChannelSendFailed(err.to_string()))?;
                        log::debug!("devices: {:?}", self.devices);
                    }
                }
                HotplugEvent::Disconnected(device_id) => {
                    log::debug!("Disconnect: {:?}", device_id);
                    self.devices.remove(&device_id);
                    let current = self.get_all(DeviceSelector::Auto).ok();
                    tx.send(current)
                        .map_err(|err| MdropError::ChannelSendFailed(err.to_string()))?;
                    log::debug!("devices: {:?}", self.devices);
                }
            }
        }

        Ok(())
    }

    pub fn detect(&self) -> MdropResult<Vec<MoondropInfo>> {
        self.devices
            .values()
            .map(|di| -> MdropResult<MoondropInfo> {
                let name = Self::device_name(di);

                let volume_payload = Self::read(di, &GET_VOLUME, ALL_PAYLOAD_LEN as u16)?;
                Self::validate_payload(&volume_payload, ALL_PAYLOAD_LEN)?;
                let vol_data = volume_payload[VOLUME_IDX];
                let vol = Volume::from_payload(vol_data);
                let bus = format!("{:02}:{:02}", di.busnum(), di.device_address());
                let data = Self::read(di, &GET_ANY, ALL_PAYLOAD_LEN as u16)?;
                MoondropInfo::new(name, bus, vol, &data)
            })
            .collect()
    }

    pub fn get_volume(&self, selector: DeviceSelector) -> MdropResult<Volume> {
        let di = self.resolve_device(&selector)?;
        let data = Self::read(di, &GET_VOLUME, ALL_PAYLOAD_LEN as u16)?;
        Self::validate_payload(&data, ALL_PAYLOAD_LEN)?;
        Ok(Volume::from_payload(data[VOLUME_IDX]))
    }

    pub fn get_filter(&self, selector: DeviceSelector) -> MdropResult<Filter> {
        Ok(self.get_all(selector)?.filter)
    }

    pub fn get_gain(&self, selector: DeviceSelector) -> MdropResult<Gain> {
        Ok(self.get_all(selector)?.gain)
    }

    pub fn get_indicator_state(&self, selector: DeviceSelector) -> MdropResult<IndicatorState> {
        Ok(self.get_all(selector)?.indicator_state)
    }

    pub fn get_all(&self, selector: DeviceSelector) -> MdropResult<MoondropInfo> {
        let di = self.resolve_device(&selector)?;
        let name = Self::device_name(di);

        let volume_payload = Self::read(di, &GET_VOLUME, ALL_PAYLOAD_LEN as u16)?;
        Self::validate_payload(&volume_payload, ALL_PAYLOAD_LEN)?;
        let vol_data = volume_payload[VOLUME_IDX];
        let vol = Volume::from_payload(vol_data);
        let bus = format!("{:02}:{:02}", di.busnum(), di.device_address());
        let data = Self::read(di, &GET_ANY, ALL_PAYLOAD_LEN as u16)?;
        MoondropInfo::new(name, bus, vol, &data)
    }

    pub fn set_gain(&mut self, selector: DeviceSelector, gain: Gain) -> MdropResult<()> {
        self.devices = Self::refresh()?;
        let di = self.resolve_device(&selector)?;
        let mut cmd = Vec::from(SET_GAIN);
        cmd.push(gain as u8);
        log::debug!("Gain Command: {:?}", cmd);
        Self::write(di, &cmd)
    }

    pub fn set_volume(&mut self, selector: DeviceSelector, level: Volume) -> MdropResult<()> {
        self.devices = Self::refresh()?;
        let di = self.resolve_device(&selector)?;
        let value = level.to_payload();
        log::debug!("Volume Level: {level} clamped: {value}");
        let mut cmd = Vec::from(SET_VOLUME);
        cmd.push(value);
        log::debug!("Volume Command: {:?}", cmd);
        Self::write(di, &cmd)
    }

    pub fn set_filter(&mut self, selector: DeviceSelector, filter: Filter) -> MdropResult<()> {
        self.devices = Self::refresh()?;
        let di = self.resolve_device(&selector)?;
        let mut cmd = Vec::from(SET_FILTER);
        cmd.push(filter as u8);
        log::debug!("Filter Command: {:?}", cmd);
        Self::write(di, &cmd)
    }

    pub fn set_indicator_state(
        &mut self,
        selector: DeviceSelector,
        indicator_state: IndicatorState,
    ) -> MdropResult<()> {
        self.devices = Self::refresh()?;
        let di = self.resolve_device(&selector)?;
        let mut cmd = Vec::from(SET_INDICATOR_STATE);
        cmd.push(indicator_state as u8);
        log::debug!("IndicatorState Command: {:?}", cmd);
        Self::write(di, &cmd)
    }

    fn read(di: &DeviceInfo, cmd: &[u8], length: u16) -> MdropResult<Vec<u8>> {
        let device = di
            .open()
            .wait()
            .map_err(|err| MdropError::UsbOpenFailed(err.to_string()))?;
        device
            .control_out(
                ControlOut {
                    control_type: nusb::transfer::ControlType::Vendor,
                    recipient: nusb::transfer::Recipient::Other,
                    request: REQUEST_ID_WRITE,
                    value: REQUEST_VALUE,
                    index: REQUEST_INDEX,
                    data: cmd,
                },
                CONTROL_TIMEOUT,
            )
            .wait()
            .map_err(|err| MdropError::UsbWriteFailed(err.to_string()))?;
        let payload = device
            .control_in(
                ControlIn {
                    control_type: nusb::transfer::ControlType::Vendor,
                    recipient: nusb::transfer::Recipient::Other,
                    request: REQUEST_ID_READ,
                    value: REQUEST_VALUE,
                    index: REQUEST_INDEX,
                    length,
                },
                CONTROL_TIMEOUT,
            )
            .wait()
            .map_err(|err| MdropError::UsbReadFailed(err.to_string()))?;
        Self::validate_payload(&payload, length as usize)?;
        Ok(payload)
    }

    fn write(di: &DeviceInfo, cmd: &[u8]) -> MdropResult<()> {
        let device = di
            .open()
            .wait()
            .map_err(|err| MdropError::UsbOpenFailed(err.to_string()))?;
        device
            .control_out(
                ControlOut {
                    control_type: nusb::transfer::ControlType::Vendor,
                    recipient: nusb::transfer::Recipient::Other,
                    request: REQUEST_ID_WRITE,
                    value: REQUEST_VALUE,
                    index: REQUEST_INDEX,
                    data: cmd,
                },
                CONTROL_TIMEOUT,
            )
            .wait()
            .map_err(|err| MdropError::UsbWriteFailed(err.to_string()))?;
        Ok(())
    }

    fn refresh() -> MdropResult<HashMap<DeviceId, DeviceInfo>> {
        nusb::list_devices()
            .wait()
            .map_err(|err| MdropError::DeviceEnumerationFailed(err.to_string()))
            .map(|devices| {
                devices
                    .filter(|d| d.vendor_id() == MOONDROP_VID)
                    .map(|d| (d.id(), d))
                    .collect()
            })
    }

    fn resolve_device(&self, selector: &DeviceSelector) -> MdropResult<&DeviceInfo> {
        match selector {
            DeviceSelector::Auto => match self.devices.len() {
                0 => Err(MdropError::NoDevice),
                1 => {
                    let (_, device) = self.devices.iter().next().ok_or(MdropError::NoDevice)?;
                    Ok(device)
                }
                count => Err(MdropError::AmbiguousDeviceSelection(count)),
            },
            DeviceSelector::Bus(bus) => {
                if !Self::is_valid_bus(bus) {
                    return Err(MdropError::InvalidBusFormat(bus.clone()));
                }

                self.devices
                    .values()
                    .find(|di| format!("{:02}:{:02}", di.busnum(), di.device_address()) == *bus)
                    .ok_or_else(|| MdropError::DeviceNotFound(bus.clone()))
            }
        }
    }

    fn is_valid_bus(bus: &str) -> bool {
        let mut parts = bus.split(':');
        let first = parts.next();
        let second = parts.next();
        let none = parts.next();

        match (first, second, none) {
            (Some(a), Some(b), None) => {
                a.len() == 2
                    && b.len() == 2
                    && a.chars().all(|c| c.is_ascii_digit())
                    && b.chars().all(|c| c.is_ascii_digit())
            }
            _ => false,
        }
    }

    fn device_name(di: &DeviceInfo) -> String {
        match di.product_string() {
            Some(name) => name.to_string(),
            None => match di.product_id() {
                DAWN_PRO_PID => "MOONDROP Dawn Pro".to_string(),
                _ => "Unknown".to_string(),
            },
        }
    }

    fn validate_payload(payload: &[u8], expected: usize) -> MdropResult<()> {
        if payload.len() < expected {
            return Err(MdropError::InvalidPayloadLength {
                expected,
                got: payload.len(),
            });
        }

        Ok(())
    }
}

impl Default for Moondrop {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct MoondropInfo {
    pub name: String,
    pub bus: String,
    pub volume: Volume,
    pub filter: Filter,
    pub gain: Gain,
    pub indicator_state: IndicatorState,
}

impl MoondropInfo {
    pub fn new(name: String, bus: String, volume: Volume, data: &[u8]) -> MdropResult<Self> {
        Moondrop::validate_payload(data, ALL_PAYLOAD_LEN)?;
        let filter = Filter::try_from(data[FILTER_IDX])?;
        let gain = Gain::try_from(data[GAIN_IDX])?;
        let state = IndicatorState::try_from(data[INDICATOR_STATE_IDX])?;
        Ok(Self {
            name,
            bus,
            volume,
            filter,
            gain,
            indicator_state: state,
        })
    }
}
