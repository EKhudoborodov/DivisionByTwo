#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_stm32::{self, gpio::{Level, Output, Speed}, into_ref, Peripheral};
use embassy_stm32::gpio::{Flex, Input, Pin, Pull, AnyPin};

pub struct Keyboard <'d, const ROWS: usize, const COLUMNS: usize> {
    columns: [Input<'d, AnyPin>; COLUMNS],
    rows: [Output<'d, AnyPin>; ROWS]
}
fn columns<'d>(pin: AnyPin) -> Input<'d, AnyPin> {
    into_ref!(pin);
    Input::new(pin, Pull::Down)
}
fn rows<'d>(pin: AnyPin) -> Output<'d, AnyPin>{
    into_ref!(pin);
    Output::new(pin, Level::High, Speed::Low)
}

impl <'d, const ROWS: usize, const COLUMNS: usize> Keyboard<'d, ROWS, COLUMNS>{
    pub fn new(col: [AnyPin; COLUMNS], row: [AnyPin; ROWS]) -> Self{
        Self { columns: col.map(columns), rows: row.map(rows)  }
    }

    fn read_keys(&mut self) -> [[u8; COLUMNS]; ROWS]{
        let mut read = [[0; COLUMNS]; ROWS];
        for i in 0..ROWS{
            self.rows[i].set_high();
            for j in 0..COLUMNS{
                read[i][j] = self.columns[j].is_high() as u8;
            }
            self.rows[i].set_low();
        }
        return read;
    }

    fn get_first_pressed(&mut self, keys: [[u8; COLUMNS]; ROWS]) -> u8{
        for i in 0..ROWS{
            for j in 0..COLUMNS{
                if(keys[i][j] == 1){
                    return (i*COLUMNS+j) as u8;
                }
            }
        }
        return 0;
    }

    pub fn get_key(&mut self) -> u8{
        let mut key: u8 = 0;
        let mut last: u8 = 0;
        loop {
            let keys = self.read_keys();
            key = self.get_first_pressed(keys);
            if key != 0 {
                break;
            }
        }
        loop {
            let keys = self.read_keys();
            last = key;
            key = self.get_first_pressed(keys);
            if key == 0 {
                break;
            }
        }
        return last;
    }
}