use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io_async::{Read, Write};

use crate::{
    hal::steal::Stealable,
    traits::{AsInput, AsIoReadWriteDevice, AsOutput},
};

// pub struct DynPin<'a, I, O> {
//     as_input: fn() -> I,
//     as_output: fn() -> O,
//     _lifetime: PhantomData<&'a mut ()>,
// }

// impl<'a, I, O> DynPin<'a, I, O> {
//     pub fn reborrow(&mut self) -> DynPin<'_, I, O> {
//         DynPin {
//             as_input: self.as_input,
//             as_output: self.as_output,
//             _lifetime: PhantomData,
//         }
//     }
// }

// impl<'a, I, O> DynPin<'a, I, O> {
//     pub const fn new<'b, T: PeripheralType + Stealable>(_pin: Peri<'b, T>) -> Self
//     where
//         'b: 'a,
//         Peri<'b, T>: AsInput<Target<'a> = I> + AsOutput<Target<'a> = O>,
//     {
//         DynPin {
//             _lifetime: PhantomData,
//             as_input: || unsafe { Self::magick::<T>() }.as_input(),
//             as_output: || unsafe { Self::magick::<T>() }.as_output(),
//         }
//     }

//     unsafe fn magick<'b, T: PeripheralType + Stealable>() -> Peri<'b, T> {
//         // We re-generate the peripheral from scratch.
//         // This is acceptable because the lifetime of Peri is bound to the lifetime of the DynPin.
//         // Also T is stealable, meaning it has no relevant data associated with it.
//         unsafe { T::steal() }
//     }
// }

// impl<'a, I, O: OutputPin> AsOutput for DynPin<'a, I, O> {
//     type Target<'b>
//         = O
//     where
//         'a: 'b,
//         Self: 'b;

//     fn as_output(&mut self) -> Self::Target<'_> {
//         (self.as_output)()
//     }
// }

// impl<'a, I: InputPin + 'a, O> AsInput for DynPin<'a, I, O> {
//     type Target<'b>
//         = I
//     where
//         'a: 'b,
//         Self: 'b;

//     fn as_input(&mut self) -> Self::Target<'_> {
//         (self.as_input)()
//     }
// }

pub struct DynThief<'a, O> {
    f: fn() -> O,
    _lifetime: PhantomData<&'a mut ()>,
}

impl<'a, O> DynThief<'a, O> {
    pub const fn new<I: Stealable + 'a>() -> Self
    where
        Peri<'a, I>: Into<O>,
    {
        Self {
            _lifetime: PhantomData,
            f: || Into::into(unsafe { I::steal() }),
        }
    }
}

impl<O> DynThief<'_, O> {
    pub fn reborrow(&mut self) -> DynThief<'_, O> {
        DynThief {
            f: self.f,
            _lifetime: PhantomData,
        }
    }
}

impl<O: OutputPin> AsOutput for DynThief<'_, O> {
    type Target<'a>
        = O
    where
        Self: 'a;

    fn as_output(&mut self) -> Self::Target<'_> {
        (self.f)()
    }
}

impl<O: InputPin> AsInput for DynThief<'_, O> {
    type Target<'a>
        = O
    where
        Self: 'a;

    fn as_input(&mut self) -> Self::Target<'_> {
        (self.f)()
    }
}

impl<O: Read + Write> AsIoReadWriteDevice for DynThief<'_, O> {
    type Target<'a>
        = O
    where
        Self: 'a;

    fn as_io_read_write(&mut self) -> Self::Target<'_> {
        (self.f)()
    }
}
