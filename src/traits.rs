//! Trait library crate similar to 'embedded-hal'

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io_async::{Read, Write};

pub trait AsInput<'a>: Sized {
    type Target: InputPin + 'a;
    fn as_input(self) -> Self::Target;
}

pub trait AsOutput<'a>: Sized {
    type Target: OutputPin + 'a;
    fn as_output(self) -> Self::Target;
}

pub trait AsIoReadWriteDevice<'a>: Sized {
    type Target: Read + Write + 'a;
    fn as_io_read_write(self) -> Self::Target;
}
