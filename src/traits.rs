//! Trait library crate similar to 'embedded-hal'

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io_async::{Read, Write};

use crate::dynamic::Owned;

pub trait AsInput<'a>: Sized {
    type Target: InputPin + 'a;
    fn as_input(value: Owned<'a, Self>) -> Self::Target;
}

pub trait AsOutput<'a>: Sized {
    type Target: OutputPin + 'a;
    fn as_output(value: Owned<'a, Self>) -> Self::Target;
}

pub trait AsIoReadWriteDevice<'a>: Sized {
    type Target: Read + Write + 'a;
    fn as_io_read_write(value: Owned<'a, Self>) -> Self::Target;
}
