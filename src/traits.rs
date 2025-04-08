//! Trait library crate similar to 'embedded-hal'

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io_async::{Read, Write};

pub trait AsInput {
    type Target<'a>: InputPin + 'a
    where
        Self: 'a;
    fn as_input(&mut self) -> Self::Target<'_>;
}

pub trait AsOutput {
    type Target<'a>: OutputPin + 'a
    where
        Self: 'a;
    fn as_output(&mut self) -> Self::Target<'_>;
}

pub trait AsIoReadWriteDevice {
    type Target<'a>: Read + Write + 'a
    where
        Self: 'a;
    fn as_io_read_write(&mut self) -> Self::Target<'_>;
}
