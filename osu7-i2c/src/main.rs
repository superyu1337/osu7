use ht16k33::i2c_mock::I2cMock;
use osu7_i2c::{Osu7Display, I2C_ADDR};

fn main() {
    /*
    let config = mcp2221::Config::default();
    let i2c = mcp2221::Handle::open_first(&config).unwrap();
    */

    // Create a mock I2C device.
    let i2c = I2cMock::new();
    let var_name = Osu7Display::new(i2c, I2C_ADDR);
    let mut display = var_name;

    display.initialize();

    display.write_buffer_integer(1234);
    display.commit_buffer();

    let buffer = display.device().display_buffer();
    println!("{:#?}", buffer);
}
