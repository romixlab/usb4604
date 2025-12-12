use anyhow::Result;
use std::thread::sleep;
use std::time::Duration;
use usb4604::{Level, Pio, Usb4604};

fn main() -> Result<()> {
    let usb4604 = Usb4604::open_auto()?;
    let mut pio0 = usb4604.output(Pio::Pio0, Level::Low)?;
    for _ in 0..5 {
        pio0.toggle()?;
        sleep(Duration::from_millis(1000));
    }
    Ok(())
}
