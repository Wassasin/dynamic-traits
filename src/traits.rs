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

pub trait AsIoReadWriteDevice: Sized {
    type Target<'a>: Read + Write + 'a
    where
        Self: 'a;
    fn as_io_read_write<'a, 'b: 'a>(value: Owned<'b, Self>) -> Self::Target<'a>;
}
