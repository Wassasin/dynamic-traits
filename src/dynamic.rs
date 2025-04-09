use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use embassy_hal_internal::{Peri, PeripheralType};
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};
use embedded_io_async::{Read, Write};

use crate::{
    consumer::Pins,
    hal::{
        gpio::{Input, Output},
        steal::Stealable,
        uart::Uart,
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

pub struct Owned<'a, T> {
    inner: T,
    _lifetime: PhantomData<&'a mut ()>,
}

impl<'a, T: 'a> Owned<'a, T> {
    pub const fn new(inner: T) -> Self {
        Owned {
            inner,
            _lifetime: PhantomData,
        }
    }

    pub fn into<U>(self) -> Owned<'a, U>
    where
        T: Into<U>,
        Self: 'a,
    {
        Owned {
            inner: self.inner.into(),
            _lifetime: PhantomData,
        }
    }
}

pub trait Reborrowable: Sized {
    fn reborrow<'a, 'b: 'a>(value: &'a mut Owned<'b, Self>) -> Owned<'a, Self>;
}

// impl<'c, T: OwnedEraseable<'c>> Reborrowable for T {
//     fn reborrow<'a, 'b: 'a>(_value: Owned<'a, Self>) -> Owned<'b, Self> {
//         unsafe { T::magick() }
//     }
// }

impl<'a, T: PeripheralType> Into<Owned<'a, Peri<'a, T>>> for Peri<'a, T> {
    fn into(self) -> Owned<'a, Peri<'a, T>> {
        Owned::new(self)
    }
}

impl<'a, T: PeripheralType> Into<Peri<'a, T>> for Owned<'a, Peri<'a, T>> {
    fn into(self) -> Peri<'a, T> {
        self.inner
    }
}

impl<'a, T> Into<DynThief<'a, T>> for Owned<'a, DynThief<'a, T>> {
    fn into(self) -> DynThief<'a, T> {
        self.inner
    }
}

impl<'a, T, U> Into<DynEither<'a, T, U>> for Owned<'a, DynEither<'a, T, U>> {
    fn into(self) -> DynEither<'a, T, U> {
        self.inner
    }
}

impl<T> Deref for Owned<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Owned<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait OwnedEraseable<'a>: Sized {
    unsafe fn magick() -> Owned<'a, Self>;
}

impl<'a, T: 'a> OwnedEraseable<'a> for Peri<'a, T>
where
    T: Stealable,
{
    unsafe fn magick() -> Owned<'a, Self> {
        Owned::new(unsafe { T::steal() })
    }
}

pub struct DynThief<'a, O> {
    f: fn() -> O,
    _lifetime: PhantomData<&'a mut ()>,
}

impl<'a, O> DynThief<'a, O> {
    pub const fn new<I: OwnedEraseable<'a>>(_i: &'a mut I) -> Self
    where
        Owned<'a, I>: Into<O>,
    {
        // Unsafe: we bind the lifetime by the argument
        unsafe { Self::new_unsafe() }
    }

    pub const unsafe fn new_unsafe<I: OwnedEraseable<'a>>() -> Self
    where
        Owned<'a, I>: Into<O>,
    {
        Self {
            _lifetime: PhantomData,
            f: || Into::into(unsafe { I::magick() }),
        }
    }

    pub fn reborrow(&mut self) -> DynThief<'_, O> {
        DynThief {
            f: self.f,
            _lifetime: PhantomData,
        }
    }

    pub fn build(self) -> Owned<'a, O> {
        Owned {
            inner: (self.f)(),
            _lifetime: PhantomData,
        }
    }
}

pub struct DynEither<'a, T, U> {
    left: DynThief<'a, T>,
    right: DynThief<'a, U>,
}

impl<'a, T, U> DynEither<'a, T, U> {
    pub fn new<I: OwnedEraseable<'a>>(_i: &'a mut I) -> Self
    where
        Owned<'a, I>: Into<T> + Into<U>,
    {
        // Unsafe: we ensure that the lifetime of left and right do not conflict.
        // We bind the lifetime with the argument.
        Self {
            left: unsafe { DynThief::new_unsafe() },
            right: unsafe { DynThief::new_unsafe() },
        }
    }

    pub fn reborrow(&mut self) -> DynEither<'_, T, U> {
        DynEither {
            left: self.left.reborrow(),
            right: self.right.reborrow(),
        }
    }

    pub fn left(self) -> DynThief<'a, T> {
        self.left
    }

    pub fn right(self) -> DynThief<'a, U> {
        self.right
    }
}

impl<'a> Into<Input<'a>> for Owned<'a, Input<'a>> {
    fn into(self) -> Input<'a> {
        self.inner
    }
}

impl<'a> Into<Output<'a>> for Owned<'a, Output<'a>> {
    fn into(self) -> Output<'a> {
        self.inner
    }
}

impl<'a> Into<Uart<'a>> for Owned<'a, Uart<'a>> {
    fn into(self) -> Uart<'a> {
        self.inner
    }
}

impl<'a, RX: 'a, TX: 'a> Into<Pins<RX, TX>> for Owned<'a, Pins<RX, TX>> {
    fn into(self) -> Pins<RX, TX> {
        Pins {
            rx: self.inner.rx,
            tx: self.inner.tx,
        }
    }
}

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
