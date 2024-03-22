#[derive(Debug)]
pub enum ColorFamily {
    DeviceGray,
    DeviceRGB,
    DeviceCMYK,
    CalGray,
    CalRGB,
    Lab,
    ICCBased,
    Separation,
    DeviceN,
    Indexed,
    Pattern,
}

pub trait Color {}
