use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ops::Not;

use super::bitreader::BitReader;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Pass,
    Horizontal,
    V,
    VR(u8),
    VL(u8),
    End,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Not for Color {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

pub fn decode_mode(bitreader: &mut BitReader) -> Option<Mode> {
    let mut v: u16 = 0;
    let mut len: u8 = 0;
    for _ in 0..12 {
        v = (v << 1) | bitreader.read_bit().unwrap() as u16;
        len += 1;
        if let Some(m) = MODE_TABLE.get(&(v, len)) {
            return Some(m.to_owned());
        }
    }
    None
}
pub fn decode_run_length(bitreader: &mut BitReader, color: Color) -> Option<u16> {
    let mut v: u16 = 0;
    let mut len: u8 = 0;
    let mut res = 0;
    match color {
        Color::Black => {
            for _ in 0..255 {
                v = (v << 1) | bitreader.read_bit().unwrap() as u16;
                len += 1;
                if let Some(rl) = BLACK_TABLE.get(&(v, len)) {
                    res += rl;
                    if rl < &64 {
                        return Some(res);
                    } else {
                        v = 0;
                        len = 0;
                    }
                }
            }
            None
        }
        Color::White => {
            for _ in 0..255 {
                v = (v << 1) | bitreader.read_bit().unwrap() as u16;
                len += 1;
                if let Some(rl) = WHITE_TABLE.get(&(v, len)) {
                    res += rl;
                    if rl < &64 {
                        return Some(res);
                    } else {
                        v = 0;
                        len = 0;
                    }
                }
            }
            None
        }
    }
}

lazy_static! {
    static ref MODE_TABLE: HashMap<(u16, u8), Mode> = {
        let mut m = HashMap::new();
        m.insert((1, 4), Mode::Pass);
        m.insert((1, 3), Mode::Horizontal);
        m.insert((1, 1), Mode::V);
        m.insert((3, 3), Mode::VR(1));
        m.insert((3, 6), Mode::VR(2));
        m.insert((3, 7), Mode::VR(3));
        m.insert((2, 3), Mode::VL(1));
        m.insert((2, 6), Mode::VL(2));
        m.insert((2, 7), Mode::VL(3));
        m.insert((1, 12), Mode::End);
        m
    };
}

lazy_static! {
    static ref WHITE_TABLE: HashMap<(u16, u8), u16> = {
        let mut m = HashMap::new();
        m.insert((53, 8), 0);
        m.insert((7, 6), 1);
        m.insert((7, 4), 2);
        m.insert((8, 4), 3);
        m.insert((11, 4), 4);
        m.insert((12, 4), 5);
        m.insert((14, 4), 6);
        m.insert((15, 4), 7);
        m.insert((19, 5), 8);
        m.insert((20, 5), 9);
        m.insert((7, 5), 10);
        m.insert((8, 5), 11);
        m.insert((8, 6), 12);
        m.insert((3, 6), 13);
        m.insert((52, 6), 14);
        m.insert((53, 6), 15);
        m.insert((42, 6), 16);
        m.insert((43, 6), 17);
        m.insert((39, 7), 18);
        m.insert((12, 7), 19);
        m.insert((8, 7), 20);
        m.insert((23, 7), 21);
        m.insert((3, 7), 22);
        m.insert((4, 7), 23);
        m.insert((40, 7), 24);
        m.insert((43, 7), 25);
        m.insert((19, 7), 26);
        m.insert((36, 7), 27);
        m.insert((24, 7), 28);
        m.insert((2, 8), 29);
        m.insert((3, 8), 30);
        m.insert((26, 8), 31);
        m.insert((27, 8), 32);
        m.insert((18, 8), 33);
        m.insert((19, 8), 34);
        m.insert((20, 8), 35);
        m.insert((21, 8), 36);
        m.insert((22, 8), 37);
        m.insert((23, 8), 38);
        m.insert((40, 8), 39);
        m.insert((41, 8), 40);
        m.insert((42, 8), 41);
        m.insert((43, 8), 42);
        m.insert((44, 8), 43);
        m.insert((45, 8), 44);
        m.insert((4, 8), 45);
        m.insert((5, 8), 46);
        m.insert((10, 8), 47);
        m.insert((11, 8), 48);
        m.insert((82, 8), 49);
        m.insert((83, 8), 50);
        m.insert((84, 8), 51);
        m.insert((85, 8), 52);
        m.insert((36, 8), 53);
        m.insert((37, 8), 54);
        m.insert((88, 8), 55);
        m.insert((89, 8), 56);
        m.insert((90, 8), 57);
        m.insert((91, 8), 58);
        m.insert((74, 8), 59);
        m.insert((75, 8), 60);
        m.insert((50, 8), 61);
        m.insert((51, 8), 62);
        m.insert((52, 8), 63);
        m.insert((27, 5), 64);
        m.insert((18, 5), 128);
        m.insert((23, 6), 192);
        m.insert((55, 7), 256);
        m.insert((54, 8), 320);
        m.insert((55, 8), 384);
        m.insert((100, 8), 448);
        m.insert((101, 8), 512);
        m.insert((104, 8), 576);
        m.insert((103, 8), 640);
        m.insert((204, 9), 704);
        m.insert((205, 9), 768);
        m.insert((210, 9), 832);
        m.insert((211, 9), 896);
        m.insert((212, 9), 960);
        m.insert((213, 9), 1024);
        m.insert((214, 9), 1088);
        m.insert((215, 9), 1152);
        m.insert((216, 9), 1216);
        m.insert((217, 9), 1280);
        m.insert((218, 9), 1344);
        m.insert((219, 9), 1408);
        m.insert((152, 9), 1472);
        m.insert((153, 9), 1536);
        m.insert((154, 9), 1600);
        m.insert((24, 6), 1664);
        m.insert((155, 9), 1728);
        m.insert((8, 11), 1792);
        m.insert((12, 11), 1856);
        m.insert((13, 11), 1920);
        m.insert((18, 12), 1984);
        m.insert((19, 12), 2048);
        m.insert((20, 12), 2112);
        m.insert((21, 12), 2176);
        m.insert((22, 12), 2240);
        m.insert((23, 12), 2304);
        m.insert((28, 12), 2368);
        m.insert((29, 12), 2432);
        m.insert((30, 12), 2496);
        m.insert((31, 12), 2560);
        m
    };
}

lazy_static! {
    static ref BLACK_TABLE: HashMap<(u16, u8), u16> = {
        let mut m = HashMap::new();
        m.insert((55, 10), 0);
        m.insert((2, 3), 1);
        m.insert((3, 2), 2);
        m.insert((2, 2), 3);
        m.insert((3, 3), 4);
        m.insert((3, 4), 5);
        m.insert((2, 4), 6);
        m.insert((3, 5), 7);
        m.insert((5, 6), 8);
        m.insert((4, 6), 9);
        m.insert((4, 7), 10);
        m.insert((5, 7), 11);
        m.insert((7, 7), 12);
        m.insert((4, 8), 13);
        m.insert((7, 8), 14);
        m.insert((24, 9), 15);
        m.insert((23, 10), 16);
        m.insert((24, 10), 17);
        m.insert((8, 10), 18);
        m.insert((103, 11), 19);
        m.insert((104, 11), 20);
        m.insert((108, 11), 21);
        m.insert((55, 11), 22);
        m.insert((40, 11), 23);
        m.insert((23, 11), 24);
        m.insert((24, 11), 25);
        m.insert((202, 12), 26);
        m.insert((203, 12), 27);
        m.insert((204, 12), 28);
        m.insert((205, 12), 29);
        m.insert((104, 12), 30);
        m.insert((105, 12), 31);
        m.insert((106, 12), 32);
        m.insert((107, 12), 33);
        m.insert((210, 12), 34);
        m.insert((211, 12), 35);
        m.insert((212, 12), 36);
        m.insert((213, 12), 37);
        m.insert((214, 12), 38);
        m.insert((215, 12), 39);
        m.insert((108, 12), 40);
        m.insert((109, 12), 41);
        m.insert((218, 12), 42);
        m.insert((219, 12), 43);
        m.insert((84, 12), 44);
        m.insert((85, 12), 45);
        m.insert((86, 12), 46);
        m.insert((87, 12), 47);
        m.insert((100, 12), 48);
        m.insert((101, 12), 49);
        m.insert((82, 12), 50);
        m.insert((83, 12), 51);
        m.insert((36, 12), 52);
        m.insert((55, 12), 53);
        m.insert((56, 12), 54);
        m.insert((39, 12), 55);
        m.insert((40, 12), 56);
        m.insert((88, 12), 57);
        m.insert((89, 12), 58);
        m.insert((43, 12), 59);
        m.insert((44, 12), 60);
        m.insert((90, 12), 61);
        m.insert((102, 12), 62);
        m.insert((103, 12), 63);
        m.insert((15, 10), 64);
        m.insert((200, 12), 128);
        m.insert((201, 12), 192);
        m.insert((91, 12), 256);
        m.insert((51, 12), 320);
        m.insert((52, 12), 384);
        m.insert((53, 12), 448);
        m.insert((108, 13), 512);
        m.insert((109, 13), 576);
        m.insert((74, 13), 640);
        m.insert((75, 13), 704);
        m.insert((76, 13), 768);
        m.insert((77, 13), 832);
        m.insert((114, 13), 896);
        m.insert((115, 13), 960);
        m.insert((116, 13), 1024);
        m.insert((117, 13), 1088);
        m.insert((118, 13), 1152);
        m.insert((119, 13), 1216);
        m.insert((82, 13), 1280);
        m.insert((83, 13), 1344);
        m.insert((84, 13), 1408);
        m.insert((85, 13), 1472);
        m.insert((90, 13), 1536);
        m.insert((91, 13), 1600);
        m.insert((100, 13), 1664);
        m.insert((101, 13), 1728);
        m.insert((8, 11), 1792);
        m.insert((12, 11), 1856);
        m.insert((13, 11), 1920);
        m.insert((18, 12), 1984);
        m.insert((19, 12), 2048);
        m.insert((20, 12), 2112);
        m.insert((21, 12), 2176);
        m.insert((22, 12), 2240);
        m.insert((23, 12), 2304);
        m.insert((28, 12), 2368);
        m.insert((29, 12), 2432);
        m.insert((30, 12), 2496);
        m.insert((31, 12), 2560);
        m
    };
}
