use osu7_i2c::{Osu7Display, I2C_ADDR};

fn main() {
    let config = mcp2221::Config::default();
    let i2c = mcp2221::Handle::open_first(&config).unwrap();

    let mut display = Osu7Display::new(i2c, I2C_ADDR);

    println!("Initializing display.");
    display.initialize();

    println!("Sleeping for 10 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(10));

    println!("Turning off display.");
    display.shutdown();
}
