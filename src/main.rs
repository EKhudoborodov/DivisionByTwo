#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::println;
use embassy_executor::Spawner;
use embassy_stm32::{
    self,
    gpio::{Level, Input, Output, Speed, Pin, AnyPin},
};
//use embassy_time::{Duration, Timer};
use tm1638::TM1638;
use keyboard::Keyboard;

use {defmt_rtt as _, panic_probe as _};

struct DivisionHelper<
    'd,
    const STB: usize,
    CLK: Pin,
    DIO: Pin,
    const ROWS: usize,
    const COLUMNS: usize,
    const BUTTONS: usize
> {
    displays: TM1638<'d, STB, CLK, DIO>,
    keyboard: Keyboard<'d, ROWS, COLUMNS>,
    keyboard_fonts: [u8; BUTTONS],
    digits: [u8; 16],
    position: i8,
    diodes: [u8; 16],
    points: [u8; 16]
}

impl <
    'd,
    const STB: usize,
    CLK: Pin,
    DIO: Pin,
    const ROWS: usize,
    const COLUMNS: usize,
    const BUTTONS: usize
> DivisionHelper <'d, STB, CLK, DIO, ROWS, COLUMNS, BUTTONS> {
    fn new(s: [AnyPin; STB], c: CLK, d: DIO, col: [AnyPin; COLUMNS], row: [AnyPin; ROWS], fonts: [u8; BUTTONS]) -> Self{
        let displays = TM1638::new(s, c, d);
        let keyboard = Keyboard::new(col, row);
        Self { displays, keyboard, digits: [10;16], keyboard_fonts: fonts, position: 15, diodes: [0; 16], points: [0; 16] }
    }

    fn reset(&mut self){
        self.displays.display_on(7);
        self.displays.clean();
        self.digits = [10; 16];
        self.position = 15;
        self.displays.write(0, "insert");
        self.displays.write(20, "number");
    }

    fn convert_to_char(&mut self, number: u8) -> char {
        match char::from_digit(number as u32, 10) {
            Some(c) => { c }
            None => { ' ' }
        }
    }

    fn insert_number(&mut self){
        let key = self.keyboard_fonts[self.keyboard.get_key() as usize];
        self.displays.clean();
        loop {
            let key = self.keyboard_fonts[self.keyboard.get_key() as usize];
            println!("{}", key);
            let c = self.convert_to_char(key);
            if c == ' ' {
                match key {
                    k if k == 12 || k == 18 => { self.reset(); }
                    13 => {
                        self.move_right();
                        self.position += 1;
                        self.digits[self.position as usize] = 10;
                    }
                    19 if self.digits[15] != 10 => {  break; }
                    _ => {}
                }
            } else if self.position > 0 {
                self.displays.set_segment(30, c, false);
                self.move_left();
                self.position -= 1;
                self.digits[15] = key;
            }
        }
        self.position = 0;
    }

    fn is_empty(&mut self) -> bool{
        for i in 0..16{
            if self.digits[i] != 10 { return false; }
        }
        return true;
    }

    fn move_left(&mut self){
        for i in self.position..15 {
            self.digits[i as usize] = self.digits[(i+1) as usize];
            let c = self.convert_to_char(self.digits[i as usize]);
            self.displays.set_segment(2*i as u8, c, false);
        }
    }

    fn move_right(&mut self){
        for i in self.position+1..16 {
            self.digits[i as usize] = self.digits[(i-1) as usize];
            let c = self.convert_to_char(self.digits[i as usize]);
            self.displays.set_segment(2*i as u8, c, false);
        }
    }

    fn show(&mut self){
        for i in 0..16{
            if self.diodes[i] == 1 {
                self.displays.set_segment((2*i+1) as u8, '8', false);
            }
        }
    }

    fn press_any_key(&mut self) -> u8{
        let key = self.keyboard_fonts[self.keyboard.get_key() as usize];
        match key {
            10 => { return 1; }
            11 => { return 2; }
            _ => { return 0; }
        }
    }

    fn first_step(&mut self){
        for i in 1..16{
            self.diodes[i] = self.digits[i-1]%2;
        }
        self.show();
    }

    fn second_step(&mut self){
        for i in 1..16{
            let c = self.convert_to_char(self.digits[i]);
            if self.diodes[i] == 0 {
                self.displays.set_segment(((i-1)*2) as u8, c, true);
                self.points[i-1] = 1;
            }
        }
    }

    fn third_step(&mut self){
        self.diodes = [0; 16];
        for i in self.position..16 {
            if self.points[15-i as usize] == 0 { break; }
            self.diodes[15-i as usize] = 1;
            self.position += 1;
        }
    }

    fn forth_step(&mut self){
        let mut cur_num = 0;
        let mut cur = 1;
        let mut step: u8 = 0;
        for i in 0..self.position {
            if self.diodes[15-i as usize]==1{
                cur_num += self.digits[15-i as usize] as u32 * cur;
                cur *= 10;
            }
        }
        cur_num /= 2;
        cur /= 10;
        while cur>0 {
            let c = self.convert_to_char((cur_num/cur) as u8);
            self.displays.set_segment((15-self.position as u8-step)*2, c, cur == 1);
            step += 1;
            cur /= 10;
        }
    }

    fn is_finished(&mut self) -> bool{
        return self.position == 15;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let stbs = [p.PB9.degrade(), p.PB8.degrade()];
    let columns = [p.PA9.degrade(), p.PA8.degrade(), p.PB15.degrade(), p.PB14.degrade()];
    let rows = [p.PA1.degrade(), p.PA2.degrade(), p.PA3.degrade(), p.PA4.degrade(), p.PA5.degrade()];
    let fonts = [10, 1, 4, 7, 14, 11, 2, 5, 8, 0, 12, 3, 6, 9, 15, 13, 16, 17, 18, 19];
    let mut helper = DivisionHelper::new(stbs, p.PB7, p.PB6, columns, rows, fonts);
    loop {
        while helper.is_empty() {
            helper.reset();
            helper.insert_number();
        }
        helper.first_step();
        helper.press_any_key();
        helper.second_step();
        helper.press_any_key();
        while !helper.is_finished() {
            helper.third_step();
            helper.press_any_key();
            helper.forth_step();
            helper.press_any_key();
        }
    }
}
