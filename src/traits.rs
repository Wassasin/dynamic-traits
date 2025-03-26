//! Trait library crate similar to 'embedded-hal'

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use embedded_hal::{
    digital::{InputPin, OutputPin},
    i2c::I2c,
};
use embedded_io_async::{Read, Write};

pub struct OwnedRef<'a, T> {
    inner: T,
    _lifetime: PhantomData<&'a mut ()>,
}

impl<'a, T> OwnedRef<'a, T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner,
            _lifetime: PhantomData,
        }
    }
}

impl<'a, T> Deref for OwnedRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> DerefMut for OwnedRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait AsInput {
    type Target<'a>: InputPin + 'a
    where
        Self: 'a;
    fn as_input<'a>(&'a mut self) -> OwnedRef<'a, Self::Target<'a>>;
}

pub trait AsOutput {
    type Target<'a>: OutputPin + 'a
    where
        Self: 'a;
    fn as_output<'a>(&'a mut self) -> OwnedRef<'a, Self::Target<'a>>;
}

pub trait AsFlex {
    type Target<'a>: InputPin + OutputPin + 'a
    where
        Self: 'a;
    fn as_flex<'a>(&'a mut self) -> OwnedRef<'a, Self::Target<'a>>;
}

pub trait AsI2cDevice {
    type Target<'a>: I2c + 'a
    where
        Self: 'a;
    fn as_i2c<'a>(&'a mut self) -> OwnedRef<'a, Self::Target<'a>>;
}

pub trait AsIoWriteDevice {
    type Target<'a>: Write + 'a
    where
        Self: 'a;
    fn as_io_write<'a>(&'a mut self) -> OwnedRef<'a, Self::Target<'a>>;
}

pub trait AsIoReadDevice {
    type Target<'a>: Read + 'a
    where
        Self: 'a;
    fn as_io_read<'a>(&'a mut self) -> OwnedRef<'a, Self::Target<'a>>;
}

pub trait AsIoReadWriteDevice {
    type Target<'a>: Read + Write + 'a
    where
        Self: 'a;
    fn as_io_read_write<'a>(&'a mut self) -> OwnedRef<'a, Self::Target<'a>>;
}
