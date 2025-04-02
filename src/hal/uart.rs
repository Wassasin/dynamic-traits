use core::{convert::Infallible, marker::PhantomData};

use crate::hal::steal::Stealable;
use embassy_hal_internal::{Peri, PeripheralType};
use embedded_io_async::ErrorType;

mod sealed {
    pub trait Instance {}
}

pub trait Instance: sealed::Instance + PeripheralType {}

pub trait TxPin<T: Instance>: PeripheralType {}
pub trait RxPin<T: Instance>: PeripheralType {}

pub struct Uart<'a> {
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Uart<'a> {
    pub fn new<T: Instance>(
        _peri: Peri<'a, T>,
        _rx: Peri<'a, impl RxPin<T>>,
        _tx: Peri<'a, impl TxPin<T>>,
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

macro_rules! impl_instance {
    ($periph:ident) => {
        impl sealed::Instance for crate::hal::peripherals::$periph {}
        impl Instance for crate::hal::peripherals::$periph {}
        impl Stealable for crate::hal::peripherals::$periph {
            unsafe fn steal<'a>() -> Peri<'a, Self> {
                unsafe { crate::hal::peripherals::$periph::steal() }
            }
        }
    };
}

impl_instance!(UART0);
impl_instance!(UART1);
impl_instance!(UART2);

impl RxPin<crate::hal::peripherals::UART0> for crate::hal::peripherals::PIN_A {}
impl TxPin<crate::hal::peripherals::UART0> for crate::hal::peripherals::PIN_B {}
impl RxPin<crate::hal::peripherals::UART1> for crate::hal::peripherals::PIN_B {}
impl TxPin<crate::hal::peripherals::UART1> for crate::hal::peripherals::PIN_C {}
impl RxPin<crate::hal::peripherals::UART2> for crate::hal::peripherals::PIN_C {}
impl TxPin<crate::hal::peripherals::UART2> for crate::hal::peripherals::PIN_D {}
