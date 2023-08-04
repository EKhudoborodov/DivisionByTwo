#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_stm32::{self, gpio::{Level, Output, Speed}, into_ref, Peripheral};
use embassy_stm32::gpio::{Flex, Input, Pin, Pull, AnyPin};
mod fonts;
use fonts::Symbols;
use fonts::Symbols::*;


pub struct TM1638 <'d, const STB: usize, CLK: Pin, DIO: Pin>{
    stb: [Output<'d, AnyPin>; STB],
    clk: Output<'d, CLK>,
    dio: Flex<'d, DIO>
}
fn init<'d>(pin: AnyPin) -> Output<'d, AnyPin>{
    into_ref!(pin);
    Output::new(pin, Level::High, Speed::Low)
}

fn convert_to_bin(n: u8) -> [u8; 8]{
    return [(n>127) as u8, (n%128>63) as u8, (n%64>31) as u8, (n%32>15) as u8, (n%16>7) as u8, (n%8>=4)as u8, (n%4>=2) as u8, n%2]
}

impl <'d, const STB: usize, CLK: Pin, DIO: Pin> TM1638 <'d, STB, CLK, DIO> {
    pub fn new(s: [AnyPin; STB], c: CLK, d: DIO) -> Self {
        let clk = Output::new(c, Level::Low, Speed::Low);
        let mut dio = Flex::new(d);
        dio.set_as_input_output(Speed::Low, Pull::Up);
        Self { stb: s.map(init), clk, dio }
    }

    fn command(&mut self, byte: [u8; 8]) {
        for i in 0..8 {
            match byte[7-i] {
                0 => { self.dio.set_low(); }
                1 => { self.dio.set_high(); }
                _ => {}
            }
            self.clk.set_high();
            self.clk.set_low();
        }
    }

    fn listen(&mut self, s: [u8; STB]) {
        for i in 0..STB {
            match s[i] {
                0 => { self.stb[i].set_high(); }
                1 => { self.stb[i].set_low(); }
                _ => {}
            }
        }
    }

    pub fn display_on(&mut self, level: u8) {
        self.listen([1; STB]);
        self.command(convert_to_bin(136 + level));
        self.listen([0; STB]);
    }

    pub fn display_off(&mut self) {
        self.listen([1; STB]);
        self.command(convert_to_bin(128));
        self.listen([0; STB]);
    }

     pub fn select_address(&mut self, mut address: u8) {
        let mut displays: [u8; STB] = [0; STB];
        displays[address as usize / 16] = 1;
        self.listen(displays);
        address %= 16;
        self.command(convert_to_bin(192 + address));
    }

    pub fn clean(&mut self) {
        self.listen([1; STB]);
        for i in 0..17 {
            self.command([0; 8]);
        }
        self.listen([0; STB]);
    }

    pub fn set_segment(&mut self, address: u8, value: char, point: bool) -> () {
        self.select_address(address);
        let mut val: Symbols = match value.to_ascii_lowercase() {
            '0' => { DIGIT_0 }
            '1' => { DIGIT_1 }
            '2' => { DIGIT_2 }
            '3' => { DIGIT_3 }
            '4' => { DIGIT_4 }
            '5' => { DIGIT_5 }
            '6' => { DIGIT_6 }
            '7' => { DIGIT_7 }
            '8' => { DIGIT_8 }
            '9' => { DIGIT_9 }
            'a' => { DIGIT_A }
            'b' => { DIGIT_b }
            'c' => { DIGIT_C }
            'd' => { DIGIT_d }
            'e' => { DIGIT_E }
            'f' => { DIGIT_F }
            'g' => { DIGIT_G }
            'h' => { DIGIT_H }
            'i' => { DIGIT_I }
            'j' => { DIGIT_J }
            'k' => { DIGIT_K }
            'l' => { DIGIT_L }
            'm' => { DIGIT_M }
            'n' => { DIGIT_N }
            'o' => { DIGIT_0 }
            'p' => { DIGIT_P }
            'q' => { DIGIT_Q }
            'r' => { DIGIT_R }
            's' => { DIGIT_5 }
            't' => { DIGIT_T }
            'u' => { DIGIT_U }
            'v' => { DIGIT_V }
            'w' => { DIGIT_W }
            'x' => { DIGIT_H }
            'y' => { DIGIT_Y }
            'z' => { DIGIT_2 }
            _ => { EMPTY }
        };
        let v = val as u8 | if point { POINT as u8 } else { 0 };
        self.command(convert_to_bin(v));
    }

    pub fn write(&mut self, mut address: u8, text: &str){
        self.select_address(address);
        for c in text.chars() {
            self.set_segment(address, c, false);
            address+=2;
            address%=16 * STB as u8;
        }
    }
}



