pub const VOLUME_MAX: u8 = 0x00;
pub const VOLUME_MIN: u8 = 0x70;

pub fn convert_volume_to_percent(value: u8) -> u32 {
    let val = value.clamp(VOLUME_MAX, VOLUME_MIN) as u32;
    (VOLUME_MIN as u32 - val) * 100 / VOLUME_MIN as u32
}

pub fn convert_volume_to_payload(value: u8) -> u8 {
    let v = (VOLUME_MIN as u32 - value as u32 * VOLUME_MIN as u32 / 100) as u8 - 1;
    v.clamp(VOLUME_MAX, VOLUME_MIN)
}
