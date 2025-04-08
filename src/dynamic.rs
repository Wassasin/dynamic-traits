use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io_async::{Read, Write};

use crate::{
    hal::steal::Stealable,
    traits::{AsInput, AsIoReadWriteDevice, AsOutput},
};

pub struct DynPin<'a, I, O> {
    as_input: fn() -> I,
    as_output: fn() -> O,
    _lifetime: PhantomData<&'a mut ()>,
}

impl<'a, I, O> DynPin<'a, I, O> {
    pub fn reborrow(&mut self) -> DynPin<'_, I, O> {
        DynPin {
            as_input: self.as_input,
            as_output: self.as_output,
            _lifetime: PhantomData,
        }
    }
}

impl<'a, I, O> DynPin<'a, I, O> {
    pub const fn new<T: PeripheralType + Stealable>(_pin: Peri<'a, T>) -> Self
    where
        Peri<'a, T>: AsInput<Target = I> + AsOutput<Target = O>,
    {
        DynPin {
            _lifetime: PhantomData,
            as_input: || unsafe { Self::magick::<T>() }.as_input(),
            as_output: || unsafe { Self::magick::<T>() }.as_output(),
        }
    }

    unsafe fn magick<T: PeripheralType + Stealable>() -> Peri<'a, T> {
        // We re-generate the peripheral from scratch.
        // This is acceptable because the lifetime of Peri is bound to the lifetime of the DynPin.
        // Also T is stealable, meaning it has no relevant data associated with it.
        unsafe { T::steal() }
    }
}

impl<'a, I, O: OutputPin + 'a> AsOutput for DynPin<'a, I, O> {
    type Target = O;

    fn as_output(self) -> Self::Target {
        (self.as_output)()
    }
}

impl<'a, I: InputPin + 'a, O> AsInput for DynPin<'a, I, O> {
    type Target = I;

    fn as_input(self) -> Self::Target {
        (self.as_input)()
    }
}

pub struct DynIoReadWrite<'a, O> {
    as_io_read_write: fn() -> O,
    _lifetime: PhantomData<&'a mut ()>,
}

impl<'a, O> DynIoReadWrite<'a, O> {
    pub const fn new<T: PeripheralType + Stealable>(_o: Peri<'a, T>) -> Self
    where
        Peri<'a, T>: AsIoReadWriteDevice<Target = O>,
    {
        Self {
            _lifetime: PhantomData,
            as_io_read_write: || unsafe { Self::magick::<T>() }.as_io_read_write(),
        }
    }

    unsafe fn magick<T: PeripheralType + Stealable>() -> Peri<'a, T> {
        // We re-generate the peripheral from scratch.
        // This is acceptable because the lifetime of Peri is bound to the lifetime of the DynPin.
        // Also T is stealable, meaning it has no relevant data associated with it.
        unsafe { T::steal() }
    }
}

impl<'a, O: Read + Write> AsIoReadWriteDevice for DynIoReadWrite<'a, O> {
    type Target = O;

    fn as_io_read_write(self) -> Self::Target {
        (self.as_io_read_write)()
    }
}
