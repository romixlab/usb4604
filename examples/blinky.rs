use anyhow::Result;
use std::thread::sleep;
use std::time::Duration;
use usb4604::{Level, Pio, Usb4604};

fn main() -> Result<()> {
    let usb4604 = Usb4604::open_auto()?;
    let mut pio1 = usb4604.gpio(Pio::Pio0)?;
    pio1.set_as_output(Level::Low)?;
    for _ in 0..5 {
        pio1.toggle()?;
        sleep(Duration::from_millis(1000));
    }
    Ok(())
}
