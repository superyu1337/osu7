use adafruit_7segment::{Index, SevenSegment};
use embedded_hal::blocking::i2c::{Write, WriteRead};
use ht16k33::{Display, HT16K33};

pub use ht16k33::i2c_mock;
pub use ht16k33::Dimming;

use std::fmt::Debug;

pub const I2C_ADDR: u8 = 0x70;

pub struct Osu7Display<I2C> {
    dev: HT16K33<I2C>,
}

impl<I2C, E> Osu7Display<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
    E: Debug
{
    pub fn new(i2c: I2C, address: u8) -> Osu7Display<I2C> {
        Self {
            dev: HT16K33::new(i2c, address),
        }
    }

    pub fn device(&mut self) -> &mut HT16K33<I2C> {
        &mut self.dev
    }

    pub fn commit_buffer(&mut self) {
        self.dev.write_display_buffer().expect("Failed to commit display buffer");
    }

    pub fn initialize(&mut self) {
        self.dev.initialize()
            .expect("Failed to initialize HT16K33");
        self.dev
            .set_display(Display::ON)
            .expect("Could not turn on the display!");
        self.dev
            .set_dimming(Dimming::BRIGHTNESS_MIN)
            .expect("Could not set dimming!");
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