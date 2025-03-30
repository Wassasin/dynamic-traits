use core::{convert::Infallible, marker::PhantomData};
use embassy_hal_internal::Peripheral;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};

mod sealed {
    pub trait Instance {}
}

pub trait Instance: sealed::Instance {}

pub struct Flex<'a> {
    _lifetime: PhantomData<&'a ()>,
}

pub struct Output<'a>(Flex<'a>);
pub struct Input<'a>(Flex<'a>);

impl<'a> Flex<'a> {
    pub fn new<T: Instance>(_pin: impl Peripheral<P = T> + 'a) -> Self {
        Self {
            _lifetime: PhantomData,
        }
    }
}

impl<'a> Output<'a> {
    pub fn new<T: Instance>(pin: impl Peripheral<P = T> + 'a) -> Self {
        Self(Flex::new(pin))
    }
}

impl<'a> Input<'a> {
    pub fn new<T: Instance>(pin: impl Peripheral<P = T> + 'a) -> Self {
        Self(Flex::new(pin))
    }
}

impl ErrorType for Flex<'_> {
    type Error = Infallible;
}

impl ErrorType for Output<'_> {
    type Error = Infallible;
}

impl ErrorType for Input<'_> {
    type Error = Infallible;
}

impl OutputPin for Output<'_> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_low()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_high()
    }
}

impl InputPin for Input<'_> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        self.0.is_high()
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.0.is_low()
    }
}

impl OutputPin for Flex<'_> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl InputPin for Flex<'_> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

macro_rules! impl_pin {
    ($pin_periph:ident) => {
        impl sealed::Instance for crate::hal::peripherals::$pin_periph {}
        impl Instance for crate::hal::peripherals::$pin_periph {}
    };
}

impl_pin!(PIN_A);
impl_pin!(PIN_B);
impl_pin!(PIN_C);
impl_pin!(PIN_D);
