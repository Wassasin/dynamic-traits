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

pub trait AsInput<'a> {
    type Target: InputPin + 'a;
    fn as_input(&'a mut self) -> OwnedRef<'a, Self::Target>;
}

pub trait AsOutput<'a> {
    type Target: OutputPin + 'a;
    fn as_output(&'a mut self) -> OwnedRef<'a, Self::Target>;
}

pub trait AsFlex<'a> {
    type Target: InputPin + OutputPin + 'a;
    fn as_flex(&'a mut self) -> OwnedRef<'a, Self::Target>;
}

pub trait AsI2cDevice<'a> {
    type Target: I2c + 'a;
    fn as_i2c(&'a mut self) -> OwnedRef<'a, Self::Target>;
}

pub trait AsIoWriteDevice<'a> {
    type Target: Write + 'a;
    fn as_io_write(&'a mut self) -> OwnedRef<'a, Self::Target>;
}

pub trait AsIoReadDevice<'a> {
    type Target: Read + 'a;
    fn as_io_read(&'a mut self) -> OwnedRef<'a, Self::Target>;
}

pub trait AsIoReadWriteDevice<'a> {
    type Target: Read + Write + 'a;
    fn as_io_read_write(&'a mut self) -> OwnedRef<'a, Self::Target>;
}
