//! Trait library crate similar to 'embedded-hal'

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io_async::{Read, Write};

use crate::dynamic::Owned;

pub trait AsInput: Sized {
    type Target<'a>: InputPin + 'a
    where
        Self: 'a;
    fn as_input(value: Owned<'_, Self>) -> Self::Target<'_>;
}

pub trait AsOutput: Sized {
    type Target<'a>: OutputPin + 'a
    where
        Self: 'a;
    fn as_output(value: Owned<'_, Self>) -> Self::Target<'_>;
}

pub trait AsIoReadWriteDevice: Sized {
    type Target<'a>: Read + Write + 'a
    where
        Self: 'a;
    fn as_io_read_write(value: Owned<'_, Self>) -> Self::Target<'_>;
}
