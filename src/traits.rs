//! Trait library crate similar to 'embedded-hal'

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io_async::{Read, Write};

pub trait AsInput: Sized {
    type Target: InputPin;
    fn as_input(self) -> Self::Target;
}

pub trait AsOutput: Sized {
    type Target: OutputPin;
    fn as_output(self) -> Self::Target;
}

pub trait AsIoReadWriteDevice: Sized {
    type Target: Read + Write;
    fn as_io_read_write(self) -> Self::Target;
}
