use anyhow::Result;
use strum::IntoEnumIterator;
use usb4604::{Pio, Usb4604};

fn main() -> Result<()> {
    let usb4604 = Usb4604::open_auto()?;
    let mut all_pins = vec![];
    for pio in Pio::iter() {
        all_pins.push(usb4604.gpio(pio)?);
    }
    println!("Pin\tMode\tOutput level\tInput level");
    for pin in &mut all_pins {
        println!(
            "{}\t{:?}\t{:?}\t{:?}",
            pin.pio().as_ref(),
            pin.mode(),
            pin.get_output_level()?,
            pin.get_input_level()?
        );
    }
    Ok(())
}
