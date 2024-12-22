use osu7_i2c::{Osu7Display, I2C_ADDR};

fn main() {
    let config = mcp2221::Config::default();
    let i2c = mcp2221::Handle::open_first(&config).unwrap();

    let var_name = Osu7Display::new(i2c, I2C_ADDR);
    let mut display = var_name;

    display.initialize();

    //display.write_buffer_integer(123);
    //display.commit_buffer();
    //display.device().set_dimming(Dimming::BRIGHTNESS_MAX).unwrap();
}
