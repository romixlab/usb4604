use anyhow::Result;
use usb4604::{I2c, Level, Pio, Usb4604};

fn main() -> Result<()> {
    let usb4604 = Usb4604::open_auto()?;

    // Enable I2C pull-up resistors after USB4604 is booted (i.e., do not tie them to 3V3, otherwise it won't boot)
    let _pull_up_en = usb4604.output(Pio::Pio9, Level::High)?;

    let mut i2c = usb4604.i2c_bridge()?;

    // let r = i2c.write(0x55, &[1, 2, 3]);
    // println!("{:?}", r);

    let mut data = [0u8; 16];
    i2c.read(0x08, &mut data)?;
    println!("{data:02x?}");

    Ok(())
}
