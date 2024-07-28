const VOLUME_MAX: u8 = 0x00;
const VOLUME_MIN: u8 = 0x70;

pub fn convert_volume(value: u8) -> u32 {
    let val = value.clamp(VOLUME_MAX, VOLUME_MIN) as u32;
    (VOLUME_MIN as u32 - val) * 100 / VOLUME_MIN as u32
}
