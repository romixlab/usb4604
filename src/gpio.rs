use crate::usb4604_reg::*;
use crate::{Error, Usb4604};
use strum::{AsRefStr, EnumIter};

/// GPIO configured as Push-Pull output.
pub struct PushPullOutput {
    flex: Flex,
}

/// GPIO configured as an Input with an optional Pull-Up or Pull-Down resistor.
pub struct Input {
    flex: Flex,
}

/// GPIO configured as an Input or Push-Pull Low output.
pub struct OpenDrainOutput {
    flex: Flex,
}

/// GPIO that can be reconfigured on the fly.
/// Supports all features of [PushPullOutput](PushPullOutput), [Input](Input) and [OpenDrainOutput](OpenDrainOutput).
///
/// Modelled after embassy Flex.
pub struct Flex {
    usb4604: Usb4604,
    pio: Pio,
    mode: Mode,
    is_out_en: bool,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Mode {
    OutputPushPull,
    OutputOpenDrain,
    Input,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Pull {
    None,
    Up,
    Down,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Level {
    Low,
    High,
}

/// Enum over all known to be working IOs.
// Can be implemented in a more abstract way, but since there are so few IOs, it does not worth it.
#[derive(Clone, Copy, EnumIter, AsRefStr, PartialEq, Debug)]
pub enum Pio {
    Pio0,
    Pio1,
    Pio3,
    Pio8,
    Pio9,
    Pio10,
    Pio19,
    Pio20,
}

impl Flex {
    pub(crate) fn init_get_mode(usb4604: Usb4604, pio: Pio) -> Result<Flex, Error> {
        let is_out = match pio {
            Pio::Pio0 => usb4604.read_reg::<Gpio0_7Dir>()?.gpio0_out_en(),
            Pio::Pio1 => usb4604.read_reg::<Gpio0_7Dir>()?.gpio1_out_en(),
            Pio::Pio3 => usb4604.read_reg::<Gpio0_7Dir>()?.gpio3_out_en(),
            Pio::Pio8 => usb4604.read_reg::<Gpio8_10Dir>()?.gpio8_out_en(),
            Pio::Pio9 => usb4604.read_reg::<Gpio8_10Dir>()?.gpio9_out_en(),
            Pio::Pio10 => usb4604.read_reg::<Gpio8_10Dir>()?.gpio10_out_en(),
            Pio::Pio19 => usb4604.read_reg::<Gpio17_20Dir>()?.gpio19_out_en(),
            Pio::Pio20 => usb4604.read_reg::<Gpio17_20Dir>()?.gpio20_out_en(),
        };
        let mode = if is_out {
            Mode::OutputPushPull
        } else {
            Mode::Input
        };
        Ok(Flex {
            usb4604,
            pio,
            mode,
            is_out_en: is_out,
        })
    }

    /// Create Flex pin assuming input mode, intended use is to immediately call one of the into_ methods.
    pub(crate) fn init_ignore_mode(usb4604: Usb4604, pio: Pio) -> Flex {
        Flex {
            usb4604,
            pio,
            mode: Mode::Input,
            is_out_en: false,
        }
    }

    /// Set initial level and put the pin into push-pull output mode.
    pub fn set_as_output(&mut self, initial: Level) -> Result<(), Error> {
        self.set_level(initial)?;
        self.set_mode(Mode::OutputPushPull)?;
        self.set_pull(Pull::None)?;
        self.mode = Mode::OutputPushPull;
        Ok(())
    }

    /// Consume self, set initial level and put the pin into push-pull output mode, return [PushPullOutput].
    pub fn into_output(mut self, initial: Level) -> Result<PushPullOutput, Error> {
        self.set_as_output(initial)?;
        Ok(PushPullOutput { flex: self })
    }

    /// Put the pin into input mode.
    ///
    /// The internal pull-up or pull-down resistor can optionally be enabled according to pull.
    pub fn set_as_input(&mut self, pull: Pull) -> Result<(), Error> {
        self.set_mode(Mode::Input)?;
        self.set_pull(pull)?;
        self.mode = Mode::Input;
        Ok(())
    }

    /// Consume self, put the pin into input mode and return [Input].
    ///
    /// The internal pull-up or pull-down resistor can optionally be enabled according to pull.
    pub fn into_input(mut self, pull: Pull) -> Result<Input, Error> {
        self.set_as_input(pull)?;
        Ok(Input { flex: self })
    }

    /// Put the pin into input + open-drain output mode.
    ///
    /// The hardware will drive the line low if you set it to low, and will leave it floating if you set it to high,
    /// in which case you can read the input to figure out whether another device is driving the line low.
    ///
    /// The internal pull-up or pull-down resistor can optionally be enabled according to pull.
    pub fn set_as_open_drain(&mut self, pull: Pull) -> Result<(), Error> {
        self.set_mode(Mode::Input)?;
        self.set_pull(pull)?;
        self.mode = Mode::OutputOpenDrain;
        Ok(())
    }

    /// Consume self, put the pin into input + open-drain output mode, return [OpenDrainOutput].
    ///
    /// The hardware will drive the line low if you set it to low, and will leave it floating if you set it to high,
    /// in which case you can read the input to figure out whether another device is driving the line low.
    ///
    /// The internal pull-up or pull-down resistor can optionally be enabled according to pull.
    pub fn into_open_drain_output(mut self, pull: Pull) -> Result<OpenDrainOutput, Error> {
        self.set_as_open_drain(pull)?;
        Ok(OpenDrainOutput { flex: self })
    }

    fn set_mode(&mut self, mode: Mode) -> Result<(), Error> {
        let out_en = matches!(mode, Mode::OutputPushPull);
        self.is_out_en = out_en;
        match self.pio {
            Pio::Pio0 => self
                .usb4604
                .modify_reg::<Gpio0_7Dir, _>(|r| r.set_gpio0_out_en(out_en)),
            Pio::Pio1 => self
                .usb4604
                .modify_reg::<Gpio0_7Dir, _>(|r| r.set_gpio1_out_en(out_en)),
            Pio::Pio3 => self
                .usb4604
                .modify_reg::<Gpio0_7Dir, _>(|r| r.set_gpio3_out_en(out_en)),
            Pio::Pio8 => self
                .usb4604
                .modify_reg::<Gpio8_10Dir, _>(|r| r.set_gpio8_out_en(out_en)),
            Pio::Pio9 => self
                .usb4604
                .modify_reg::<Gpio8_10Dir, _>(|r| r.set_gpio9_out_en(out_en)),
            Pio::Pio10 => self
                .usb4604
                .modify_reg::<Gpio8_10Dir, _>(|r| r.set_gpio10_out_en(out_en)),
            Pio::Pio19 => self
                .usb4604
                .modify_reg::<Gpio17_20Dir, _>(|r| r.set_gpio19_out_en(out_en)),
            Pio::Pio20 => self
                .usb4604
                .modify_reg::<Gpio17_20Dir, _>(|r| r.set_gpio20_out_en(out_en)),
        }?;
        Ok(())
    }

    /// Returns current pin mode.
    /// When created as Flex, pin mode is read out from the IC, which persists until hub reset.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Returns GPIO number that this Flex is using.
    pub fn pio(&self) -> Pio {
        self.pio
    }

    /// Set the output level.
    pub fn set_level(&mut self, level: Level) -> Result<(), Error> {
        if self.mode == Mode::OutputOpenDrain {
            if level == Level::High {
                self.set_mode(Mode::Input)?;
                return Ok(());
            } else {
                self.set_mode(Mode::OutputPushPull)?;
            }
        }
        let is_high = matches!(level, Level::High);
        match self.pio {
            Pio::Pio0 => self
                .usb4604
                .modify_reg::<Gpio0_7Output, _>(|r| r.set_gpio0_out(is_high)),
            Pio::Pio1 => self
                .usb4604
                .modify_reg::<Gpio0_7Output, _>(|r| r.set_gpio1_out(is_high)),
            Pio::Pio3 => self
                .usb4604
                .modify_reg::<Gpio0_7Output, _>(|r| r.set_gpio3_out(is_high)),
            Pio::Pio8 => self
                .usb4604
                .modify_reg::<Gpio8_10Output, _>(|r| r.set_gpio8_out(is_high)),
            Pio::Pio9 => self
                .usb4604
                .modify_reg::<Gpio8_10Output, _>(|r| r.set_gpio9_out(is_high)),
            Pio::Pio10 => self
                .usb4604
                .modify_reg::<Gpio8_10Output, _>(|r| r.set_gpio10_out(is_high)),
            Pio::Pio19 => self
                .usb4604
                .modify_reg::<Gpio17_20Output, _>(|r| r.set_gpio19_out(is_high)),
            Pio::Pio20 => self
                .usb4604
                .modify_reg::<Gpio17_20Output, _>(|r| r.set_gpio20_out(is_high)),
        }?;
        Ok(())
    }

    /// Set the output as high.
    /// If the pin is configured as open-drain, then it will be reconfigured to input.
    pub fn set_high(&mut self) -> Result<(), Error> {
        self.set_level(Level::High)
    }

    /// Set the output as low.
    pub fn set_low(&mut self) -> Result<(), Error> {
        self.set_level(Level::Low)
    }

    /// Toggle the output level.
    pub fn toggle(&mut self) -> Result<(), Error> {
        if self.get_output_level()? == Level::Low {
            self.set_level(Level::High)
        } else {
            self.set_level(Level::Low)
        }
    }

    /// Get output level, previously set with [set_level](Self::set_level).
    pub fn get_output_level(&self) -> Result<Level, Error> {
        let is_high = match self.pio {
            Pio::Pio0 => self.usb4604.read_reg::<Gpio0_7Output>()?.gpio0_out(),
            Pio::Pio1 => self.usb4604.read_reg::<Gpio0_7Output>()?.gpio1_out(),
            Pio::Pio3 => self.usb4604.read_reg::<Gpio0_7Output>()?.gpio3_out(),
            Pio::Pio8 => self.usb4604.read_reg::<Gpio8_10Output>()?.gpio8_out(),
            Pio::Pio9 => self.usb4604.read_reg::<Gpio8_10Output>()?.gpio9_out(),
            Pio::Pio10 => self.usb4604.read_reg::<Gpio8_10Output>()?.gpio10_out(),
            Pio::Pio19 => self.usb4604.read_reg::<Gpio17_20Output>()?.gpio19_out(),
            Pio::Pio20 => self.usb4604.read_reg::<Gpio17_20Output>()?.gpio20_out(),
        };
        let level = if is_high { Level::High } else { Level::Low };
        Ok(level)
    }

    /// Returns true, if output was previously set to high with [set_level](Self::set_level).
    pub fn is_set_high(&self) -> Result<bool, Error> {
        Ok(self.get_output_level()? == Level::High)
    }

    /// Returns true, if output was previously set to low with [set_level](Self::set_level).
    pub fn is_set_low(&self) -> Result<bool, Error> {
        Ok(self.get_output_level()? == Level::Low)
    }

    /// Get pin input level.
    pub fn get_input_level(&self) -> Result<Level, Error> {
        let is_high = match self.pio {
            Pio::Pio0 => self.usb4604.read_reg::<Gpio0_7Input>()?.gpio0_in(),
            Pio::Pio1 => self.usb4604.read_reg::<Gpio0_7Input>()?.gpio1_in(),
            Pio::Pio3 => self.usb4604.read_reg::<Gpio0_7Input>()?.gpio3_in(),
            Pio::Pio8 => self.usb4604.read_reg::<Gpio8_10Input>()?.gpio8_in(),
            Pio::Pio9 => self.usb4604.read_reg::<Gpio8_10Input>()?.gpio9_in(),
            Pio::Pio10 => self.usb4604.read_reg::<Gpio8_10Input>()?.gpio10_in(),
            Pio::Pio19 => self.usb4604.read_reg::<Gpio17_20Input>()?.gpio19_in(),
            Pio::Pio20 => self.usb4604.read_reg::<Gpio17_20Input>()?.gpio20_in(),
        };
        let level = if is_high { Level::High } else { Level::Low };
        Ok(level)
    }

    /// Returns true if pin input level is high.
    pub fn is_high(&self) -> Result<bool, Error> {
        Ok(self.get_input_level()? == Level::High)
    }

    /// Returns true if pin input level is low.
    pub fn is_low(&self) -> Result<bool, Error> {
        Ok(self.get_input_level()? == Level::High)
    }

    /// Enable or disable pull-up or pull-down resistor.
    pub fn set_pull(&mut self, pull: Pull) -> Result<(), Error> {
        let (pull_up, pull_down) = match pull {
            Pull::None => (false, false),
            Pull::Up => (true, false),
            Pull::Down => (false, true),
        };
        match self.pio {
            Pio::Pio0 => {
                self.usb4604
                    .modify_reg::<Gpio0_7PullUp, _>(|r| r.set_gpio0_pu(pull_up))?;
                self.usb4604
                    .modify_reg::<Gpio0_7PullDown, _>(|r| r.set_gpio0_pd(pull_down))?;
            }
            Pio::Pio1 => {
                self.usb4604
                    .modify_reg::<Gpio0_7PullUp, _>(|r| r.set_gpio1_pu(pull_up))?;
                self.usb4604
                    .modify_reg::<Gpio0_7PullDown, _>(|r| r.set_gpio1_pd(pull_down))?;
            }
            Pio::Pio3 => {
                self.usb4604
                    .modify_reg::<Gpio0_7PullUp, _>(|r| r.set_gpio3_pu(pull_up))?;
                self.usb4604
                    .modify_reg::<Gpio0_7PullDown, _>(|r| r.set_gpio3_pd(pull_down))?;
            }
            Pio::Pio8 => {
                self.usb4604
                    .modify_reg::<Gpio8_10PullUp, _>(|r| r.set_gpio8_pu(pull_up))?;
                self.usb4604
                    .modify_reg::<Gpio8_10PullDown, _>(|r| r.set_gpio8_pd(pull_down))?;
            }
            Pio::Pio9 => {
                self.usb4604
                    .modify_reg::<Gpio8_10PullUp, _>(|r| r.set_gpio9_pu(pull_up))?;
                self.usb4604
                    .modify_reg::<Gpio8_10PullDown, _>(|r| r.set_gpio9_pd(pull_down))?;
            }
            Pio::Pio10 => {
                self.usb4604
                    .modify_reg::<Gpio8_10PullUp, _>(|r| r.set_gpio10_pu(pull_up))?;
                self.usb4604
                    .modify_reg::<Gpio8_10PullDown, _>(|r| r.set_gpio10_pd(pull_down))?;
            }
            Pio::Pio19 => {
                self.usb4604
                    .modify_reg::<Gpio17_20PullUp, _>(|r| r.set_gpio19_pu(pull_up))?;
                self.usb4604
                    .modify_reg::<Gpio17_20PullDown, _>(|r| r.set_gpio19_pd(pull_down))?;
            }
            Pio::Pio20 => {
                self.usb4604
                    .modify_reg::<Gpio17_20PullUp, _>(|r| r.set_gpio20_pu(pull_up))?;
                self.usb4604
                    .modify_reg::<Gpio17_20PullDown, _>(|r| r.set_gpio20_pd(pull_down))?;
            }
        }
        Ok(())
    }
}

impl PushPullOutput {
    /// Set the output as high.
    pub fn set_high(&mut self) -> Result<(), Error> {
        self.flex.set_high()
    }

    /// Set the output as low.
    pub fn set_low(&mut self) -> Result<(), Error> {
        self.flex.set_low()
    }

    /// Toggle the output level.
    pub fn toggle(&mut self) -> Result<(), Error> {
        self.flex.toggle()
    }

    /// Set the output level.
    pub fn set_level(&mut self, level: Level) -> Result<(), Error> {
        self.flex.set_level(level)
    }

    /// Get previously set output level.
    pub fn level(&self) -> Result<Level, Error> {
        self.flex.get_output_level()
    }

    /// Returns GPIO number that this output is using.
    pub fn pio(&self) -> Pio {
        self.flex.pio
    }
}

impl Input {
    /// Returns true if pin input level is high.
    pub fn is_high(&self) -> Result<bool, Error> {
        self.flex.is_high()
    }

    /// Returns true if pin input level is low.
    pub fn is_low(&self) -> Result<bool, Error> {
        self.flex.is_low()
    }

    /// Returns GPIO number that this input is using.
    pub fn pio(&self) -> Pio {
        self.flex.pio
    }

    /// Enable or disable pull-up or pull-down resistor.
    pub fn set_pull(&mut self, pull: Pull) -> Result<(), Error> {
        self.flex.set_pull(pull)
    }
}

impl OpenDrainOutput {
    /// Set the pin as input, level will depend on internal or external pull-up resistor.
    pub fn set_high_z(&mut self) -> Result<(), Error> {
        self.flex.set_high()
    }

    /// Set the output as low.
    pub fn set_low(&mut self) -> Result<(), Error> {
        self.flex.set_low()
    }

    /// Toggle the output level between high-z and low.
    pub fn toggle(&mut self) -> Result<(), Error> {
        self.flex.toggle()
    }

    /// Set the output level.
    pub fn set_level(&mut self, level: Level) -> Result<(), Error> {
        self.flex.set_level(level)
    }

    /// If previously set as low, return Level::Low, otherwise read input level and return it.
    pub fn level(&self) -> Result<Level, Error> {
        if self.flex.is_out_en {
            Ok(Level::Low)
        } else {
            self.flex.get_input_level()
        }
    }

    /// Returns GPIO number that this open drain pin is using.
    pub fn pio(&self) -> Pio {
        self.flex.pio
    }
}
