use core::{convert::Infallible, marker::PhantomData};

use embassy_hal_internal::Peripheral;
use embedded_hal::i2c::ErrorType;

mod sealed {
    pub trait Instance {}
}

pub trait Instance: sealed::Instance {}

pub struct I2c<'a> {
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> I2c<'a> {
    pub fn new<T: Instance>(_peri: impl Peripheral<P = T> + 'a) -> Self {
        Self {
            _lifetime: PhantomData,
        }
    }
}

impl ErrorType for I2c<'_> {
    type Error = Infallible;
}

impl embedded_hal_async::i2c::I2c for I2c<'_> {
    async fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl sealed::Instance for crate::hal::peripherals::I2C0 {}
impl Instance for crate::hal::peripherals::I2C0 {}
