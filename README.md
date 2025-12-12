# USB4604 HAL

> Hardware abstraction layer for USB4604 hub.

Supported features:

* [x] GPIO control
* [x] I2C master

Unsupported features:

* SPI master
* USART

## How it works

Hub IC exposes one more USB device, referred to as "feature controller" in the docs. USB control transfers are used to
access registers, which in turn can control GPIOs, I2C master controller and other things.

This crate uses awesome `nusb` to interact with the USB.

## Caveats

### GPIO

Not all GPIOs are actually working, most of the pins are multi-function, and when by default a pin is doing something
else
it most likely won't be controllable. Documentation is rather vague on this, referring to some pins as reconfigurable,
but in practice not all registers are writeable from the USB interface.

List of verified to be working GPIO pins:

* PIO0
* PIO1
* PIO3
* PIO8
* PIO10
* PIO19
* PIO20

### I2C

In order for the hub to boot properly, SCL and SDA must be held low during power-on, otherwise it will wait forever
for configuration over I2C and won't appear on USB at all. Recommended way is to use one of the GPIOs to enable
pull-up resistors.

## Related ICs

According to the documentation, USB2532, USB2533, USB2534, USB3613, USB3813, USB4624 are very similar to USB4604.
They can probably be supported rather easily, but there was no need as for now I'm only using USB4604 in my projects.
Leaving this list here to help search engines.

## Documentation

* [GPIO Register docs: AN1940](https://ww1.microchip.com/downloads/aemDocuments/documents/OTH/ApplicationNotes/ApplicationNotes/00001940C.pdf)
* [Register docs](https://ww1.microchip.com/downloads/aemDocuments/documents/OTH/ApplicationNotes/ApplicationNotes/00001801C.pdf)
