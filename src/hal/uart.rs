use core::{convert::Infallible, marker::PhantomData};

use embassy_hal_internal::Peripheral;
use embedded_io_async::ErrorType;

mod sealed {
    pub trait Instance {}
}

pub trait Instance: sealed::Instance {}

pub trait TxPin<T: Instance> {}
pub trait RxPin<T: Instance> {}

pub struct Uart<'a> {
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Uart<'a> {
    pub fn new<T: Instance>(
        _peri: impl Peripheral<P = T> + 'a,
        _rx: impl Peripheral<P = impl RxPin<T>>,
        _tx: impl Peripheral<P = impl TxPin<T>>,
    ) -> Self {
        Self {
            _lifetime: PhantomData,
        }
    }
}

impl ErrorType for Uart<'_> {
    type Error = Infallible;
}

impl embedded_io_async::Read for Uart<'_> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(buf.len())
    }
}

impl embedded_io_async::Write for Uart<'_> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(buf.len())
    }
}

impl sealed::Instance for crate::hal::peripherals::UART0 {}
impl Instance for crate::hal::peripherals::UART0 {}

impl RxPin<crate::hal::peripherals::UART0> for crate::hal::peripherals::PIN_A {}
impl TxPin<crate::hal::peripherals::UART0> for crate::hal::peripherals::PIN_B {}
