pub mod cal_gray;
pub mod cal_rgb;
pub mod device_cmyk;
pub mod device_gray;
pub mod device_rgb;
pub mod devicen;
pub mod iccbased;
pub mod indexed;
pub mod lab;
pub mod pattern;
pub mod separation;

mod common;

#[derive(Debug)]
pub enum ColorSpace {
    DeviceGray(device_gray::DeviceGray),
    DeviceRGB(device_rgb::DeviceRGB),
    DeviceCMYK(device_cmyk::DeviceCMYK),
    CalGray(cal_gray::CalGray),
    CalRGB(cal_rgb::CalRGB),
    Lab(lab::Lab),
    ICCBased(iccbased::IccBased),
    Separation(separation::Separation),
    DeviceN(devicen::DeviceN),
    Indexed(indexed::Indexed),
    Pattern(pattern::Pattern),
}
