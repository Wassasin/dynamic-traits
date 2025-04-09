use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use embassy_hal_internal::{Peri, PeripheralType};
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};
use embedded_io_async::{Read, Write};

use crate::{
    hal::{
        gpio::{Input, Output},
        steal::Stealable,
    },
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
    pub const fn new<I: Stealable + 'a>(_i: Peri<'a, I>) -> Self
    where
        Peri<'a, I>: Into<O>,
    {
        Self {
            _lifetime: PhantomData,
            f: || Into::into(unsafe { I::steal() }),
        }
    }

    pub fn reborrow(&mut self) -> DynThief<'_, O> {
        DynThief {
            f: self.f,
            _lifetime: PhantomData,
        }
    }

    pub fn build(&mut self) -> DynThiefRef<'a, O> {
        DynThiefRef {
            inner: (self.f)(),
            _lifetime: PhantomData,
        }
    }
}

pub struct DynThiefRef<'a, O> {
    inner: O,
    _lifetime: PhantomData<&'a mut ()>,
}

impl<O> Deref for DynThiefRef<'_, O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<O> DerefMut for DynThiefRef<'_, O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<O> DynThiefRef<'_, O> {
    pub unsafe fn into_inner(self) -> O {
        self.inner
    }
}

pub struct DynEither<'a, T, U> {
    left: DynThief<'a, T>,
    right: DynThief<'a, U>,
}

impl<'a, T, U> DynEither<'a, T, U> {
    pub fn new<I: Stealable + 'a>(i: Peri<'a, I>) -> Self
    where
        Peri<'a, I>: Into<T> + Into<U>,
    {
        // Unsafe: we ensure that the lifetime of left and right do not conflict.
        Self {
            left: DynThief::new(unsafe { i.clone_unchecked() }),
            right: DynThief::new(i),
        }
    }

    pub fn reborrow(&mut self) -> DynEither<'_, T, U> {
        DynEither {
            left: self.left.reborrow(),
            right: self.right.reborrow(),
        }
    }

    pub fn build_left(&mut self) -> DynThiefRef<'_, T> {
        self.left.build()
    }

    pub fn build_right(&mut self) -> DynThiefRef<'_, U> {
        self.right.build()
    }
}

impl<'a> Into<Input<'a>> for DynThiefRef<'a, Input<'a>> {
    fn into(self) -> Input<'a> {
        self.inner
    }
}

impl<'a> Into<Output<'a>> for DynThiefRef<'a, Output<'a>> {
    fn into(self) -> Output<'a> {
        self.inner
    }
}

// impl<O: ErrorType> ErrorType for DynThiefRef<'_, O> {
//     type Error = O::Error;
// }

// impl<O: OutputPin> OutputPin for DynThiefRef<'_, O> {
//     fn set_low(&mut self) -> Result<(), Self::Error> {
//         self.inner.set_low()
//     }

//     fn set_high(&mut self) -> Result<(), Self::Error> {
//         self.inner.set_high()
//     }
// }

// impl<O: InputPin> InputPin for DynThiefRef<'_, O> {
//     fn is_high(&mut self) -> Result<bool, Self::Error> {
//         self.inner.is_high()
//     }

//     fn is_low(&mut self) -> Result<bool, Self::Error> {
//         self.inner.is_low()
//     }
// }

// impl<O: OutputPin> AsOutput for DynThief<'_, O> {
//     type Target<'a>
//         = O
//     where
//         Self: 'a;

//     fn as_output(&mut self) -> Self::Target<'_> {
//         (self.f)()
//     }
// }

// impl<O: InputPin> AsInput for DynThief<'_, O> {
//     type Target<'a>
//         = O
//     where
//         Self: 'a;

//     fn as_input(&mut self) -> Self::Target<'_> {
//         (self.f)()
//     }
// }

// impl<O: Read + Write> AsIoReadWriteDevice for DynThief<'_, O> {
//     type Target<'a>
//         = O
//     where
//         Self: 'a;

//     fn as_io_read_write(&mut self) -> Self::Target<'_> {
//         (self.f)()
//     }
// }
