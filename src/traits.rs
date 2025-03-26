//! Trait library crate similar to 'embedded-hal'

use embedded_hal::{
    digital::{InputPin, OutputPin},
    i2c::I2c,
};
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

pub trait AsFlex {
    type Target<'a>: InputPin + OutputPin + 'a
    where
        Self: 'a;
    fn as_flex(&mut self) -> Self::Target<'_>;
}

pub trait AsI2cDevice {
    type Target<'a>: I2c + 'a
    where
        Self: 'a;
    fn as_i2c(&mut self) -> Self::Target<'_>;
}

pub trait AsIoWriteDevice {
    type Target<'a>: Write + 'a
    where
        Self: 'a;
    fn as_io_write(&mut self) -> Self::Target<'_>;
}

pub trait AsIoReadDevice {
    type Target<'a>: Read + 'a
    where
        Self: 'a;
    fn as_io_read(&mut self) -> Self::Target<'_>;
}

pub trait AsIoReadWriteDevice {
    type Target<'a>: Read + Write + 'a
    where
        Self: 'a;
    fn as_io_read_write(&mut self) -> Self::Target<'_>;
}
