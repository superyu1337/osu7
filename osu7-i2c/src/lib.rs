use adafruit_7segment::{Index, SevenSegment};
use embedded_hal::blocking::i2c::{Write, WriteRead};
use ht16k33::DisplayData;
use ht16k33::HT16K33;

pub use adafruit_7segment::AsciiChar;
pub use ht16k33::i2c_mock;
pub use ht16k33::Dimming;
pub use ht16k33::Display;

use std::fmt::Debug;

pub const I2C_ADDR: u8 = 0x70;

pub struct Osu7Display<I2C> {
    dev: HT16K33<I2C>,
    old_buffer: [DisplayData; 16],
}

impl<I2C, E> Osu7Display<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
    E: Debug,
{
    pub fn new(i2c: I2C, address: u8) -> Osu7Display<I2C> {
        Self {
            dev: HT16K33::new(i2c, address),
            old_buffer: [DisplayData::empty(); 16],
        }
    }

    pub fn destroy(self) {
        self.dev.destroy();
    }

    pub fn shutdown(mut self) {
        self.dev
            .set_display(Display::OFF)
            .expect("Could not turn off the display");
        self.dev.destroy();
    }

    pub fn device(&mut self) -> &mut HT16K33<I2C> {
        &mut self.dev
    }

    pub fn commit_buffer(&mut self) -> Result<(), E> {
        let new_buffer = self.dev.display_buffer();

        if self.old_buffer != *self.dev.display_buffer() {
            self.old_buffer = *new_buffer;
            return self.dev.write_display_buffer();
        }

        Ok(())
    }

    pub fn write_buffer_osu7(&mut self) -> Result<(), E> {
        self.dev
            .update_buffer_with_char(Index::One, AsciiChar::new('0'))
            .expect("Failed to set 0");
        self.dev
            .update_buffer_with_char(Index::Two, AsciiChar::new('S'))
            .expect("Failed to set S");
        self.dev
            .update_buffer_with_char(Index::Three, AsciiChar::new('U'))
            .expect("Failed to set U");
        self.dev
            .update_buffer_with_char(Index::Four, AsciiChar::new('7'))
            .expect("Failed to set 7");
        Ok(())
    }

    pub fn initialize(&mut self) {
        self.dev.initialize().expect("Failed to initialize HT16K33");
        self.dev
            .set_display(Display::ON)
            .expect("Could not turn on the display");
        self.dev
            .set_dimming(Dimming::BRIGHTNESS_MIN)
            .expect("Could not set dimming");

        self.write_buffer_osu7()
            .expect("Failed to write default buffer");
        self.commit_buffer().expect("Failed to commit buffer");
    }

    pub fn write_chars(&mut self, chars: [Option<AsciiChar>; 4]) {
        if let Some(character) = chars[0] {
            self.dev
                .update_buffer_with_char(Index::One, character)
                .expect("Failed to set char");
        }

        if let Some(character) = chars[1] {
            self.dev
                .update_buffer_with_char(Index::Two, character)
                .expect("Failed to set char");
        }

        if let Some(character) = chars[2] {
            self.dev
                .update_buffer_with_char(Index::Three, character)
                .expect("Failed to set char");
        }

        if let Some(character) = chars[3] {
            self.dev
                .update_buffer_with_char(Index::Four, character)
                .expect("Failed to set char");
        }
    }

    pub fn write_buffer_float(&mut self, float: f32) {
        self.dev.clear_display_buffer();
        self.dev
            .update_buffer_with_float(Index::One, float, 2, 10)
            .expect("Failed to write float");
    }

    /// Write a 4-digit integer into the display buffer.
    pub fn write_buffer_integer(&mut self, number: u32) {
        self.dev.clear_display_buffer();
        if number > 9999 {
            return;
        }

        let mut num_chars: Vec<Option<u8>> = number
            .to_string()
            .chars()
            .map(|c| Some(c.to_digit(10).unwrap() as u8))
            .collect();

        num_chars.reverse();

        while num_chars.len() < 4 {
            num_chars.push(None);
        }

        num_chars.reverse();

        if let Some(v) = num_chars[0] {
            self.dev.update_buffer_with_digit(Index::One, v);
        }

        if let Some(v) = num_chars[1] {
            self.dev.update_buffer_with_digit(Index::Two, v);
        }

        if let Some(v) = num_chars[2] {
            self.dev.update_buffer_with_digit(Index::Three, v);
        }

        if let Some(v) = num_chars[3] {
            self.dev.update_buffer_with_digit(Index::Four, v);
        }
    }
}
