use embassy_hal_internal::Peri;

use crate::hal::gpio::{Input, Instance, Output};
use crate::traits::{AsInput, AsOutput};

impl<T: Instance> AsOutput for Peri<'_, T> {
    type Target<'a>
        = Output<'a>
    where
        Self: 'a;

    fn as_output(&mut self) -> Self::Target<'_> {
        Output::new(self.reborrow())
    }
}

impl<T: Instance> AsInput for Peri<'_, T> {
    type Target<'a>
        = Input<'a>
    where
        Self: 'a;

    fn as_input(&mut self) -> Self::Target<'_> {
        Input::new(self.reborrow())
    }
}
