use std::thread::sleep;
use std::time::Duration;

use rusb::{DeviceHandle, UsbContext};

const MOONDROP_VID: u16 = 0x2fc6;
const DAWN_PRO_PID: u16 = 0xf06a;

const REQUEST_INDEX: u16 = 0x09A0;
const REQUEST_VALUE: u16 = 0x0000;

const REQUEST_ID_WRITE: u8 = 0xA0;
const REQUEST_ID_READ: u8 = 0xA1;

const REQUEST_TYPE_WRITE: u8 = 0x43;
const REQUEST_TYPE_READ: u8 = 0xC3;

pub fn usb_read<T: UsbContext>(handle: &DeviceHandle<T>, cmd: &[u8], data: &mut [u8]) {
    let _ = handle
        .write_control(
            REQUEST_TYPE_WRITE,
            REQUEST_ID_WRITE,
            REQUEST_VALUE,
            REQUEST_INDEX,
            cmd,
            Duration::from_millis(100),
        )
        .unwrap();
    sleep(Duration::from_millis(100));
    let _ = handle
        .read_control(
            REQUEST_TYPE_READ,
            REQUEST_ID_READ,
            REQUEST_VALUE,
            REQUEST_INDEX,
            data,
            Duration::from_millis(100),
        )
        .unwrap();
}

pub fn usb_write<T: UsbContext>(handle: &DeviceHandle<T>, cmd: &[u8]) {
    let res = handle
        .write_control(
            REQUEST_TYPE_WRITE,
            REQUEST_ID_WRITE,
            REQUEST_VALUE,
            REQUEST_INDEX,
            cmd,
            Duration::from_millis(100),
        )
        .unwrap();
    println!("Wrote {} bytes", res);
}

pub fn set<T: UsbContext>(context: &mut T, cmd: &[u8]) {
    let devices = context.devices().unwrap();

    devices
        .iter()
        .filter(|device| {
            let device_desc = device.device_descriptor().unwrap();
            device_desc.vendor_id() == MOONDROP_VID
        })
        .for_each(|device| {
            if let Ok(dev) = device.open() {
                usb_write(&dev, cmd);
            }
        });
}

pub fn get<T: UsbContext>(context: &mut T, cmd: &[u8], data: &mut [u8]) {
    let devices = context.devices().unwrap();

    devices
        .iter()
        .filter(|device| {
            let device_desc = device.device_descriptor().unwrap();
            device_desc.vendor_id() == MOONDROP_VID
        })
        .for_each(|device| {
            let device_desc = device.device_descriptor().unwrap();
            let product_name = if let Ok(dev) = device.open() {
                usb_read(&dev, cmd, data);
                dev.read_product_string_ascii(&device_desc).unwrap()
            } else {
                "Unknown".to_string()
            };

            println!(
                "Bus: {:03} Device {:03} ID: {:04x}:{:04x} Name: {} Port Number: {}",
                device.bus_number(),
                device.address(),
                device_desc.vendor_id(),
                device_desc.product_id(),
                product_name,
                device.port_number()
            );
        });
}
pub fn detect<T: UsbContext>(context: &mut T) -> Vec<String> {
    let devices = context.devices().unwrap();

    devices
        .iter()
        .filter(|device| {
            let device_desc = device.device_descriptor().unwrap();
            device_desc.vendor_id() == MOONDROP_VID
        })
        .map(
            |device| match device.device_descriptor().unwrap().product_id() {
                DAWN_PRO_PID => "Moondrop Dawn Pro".to_string(),
                _ => "Unknown".to_string(),
            },
        )
        .collect()
}
