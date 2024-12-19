use adafruit_7segment::{Index, SevenSegment};
use embedded_hal::blocking::i2c::{Write, WriteRead};
use ht16k33::{Dimming, Display, HT16K33};

pub use ht16k33::i2c_mock;

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
        if number > 9999 {
            return;
        }

        let string = format!("{:04}", number);
        let chars: Vec<u8> = string
            .chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect::<Vec<u8>>();

        self.dev.update_buffer_with_digit(Index::One, chars[0]);
        self.dev.update_buffer_with_digit(Index::Two, chars[1]);
        self.dev.update_buffer_with_digit(Index::Three, chars[2]);
        self.dev.update_buffer_with_digit(Index::Four, chars[3]);
    }
}