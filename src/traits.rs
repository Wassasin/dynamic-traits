//! Trait library crate similar to 'embedded-hal'

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io_async::{Read, Write};

pub trait AsInput {
    type Target: InputPin;
    fn as_input(self) -> Self::Target;
}

pub trait AsOutput {
    type Target: OutputPin;
    fn as_output(self) -> Self::Target;
}

pub trait AsIoReadWriteDevice {
    type Target: Read + Write;
    fn as_io_read_write(self) -> Self::Target;
}

impl<T> AsIoReadWriteDevice for T
where
    T: Read + Write,
{
    type Target = T;

    fn as_io_read_write(self) -> Self::Target {
        self
    }
}
