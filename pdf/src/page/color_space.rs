// PDF3200: Table 62 – Colour Space Families
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

#[derive(Debug)]
pub struct ColorSpace {
    family: ColorFamily,
    values: Vec<i32>,
}
