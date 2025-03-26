use dynamic_traits::{
    consumer::{self, AsPins},
    hal::{
        Peripherals,
        gpio::{self, Input, Output},
        peripherals::{PIN_A, PIN_B},
        uart::Uart,
    },
    traits::AsOutput,
};
use embedded_hal::digital::OutputPin;

// impl AsOutput for impl gpio::Instance {
//     type Target;

//     fn as_output<'a>(&'a mut self) -> dynamic_traits::traits::OwnedRef<'a, Self::Target> {
//         todo!()
//     }
// }

struct PlatformManager<'a> {
    pins: consumer::Pins<'a, PIN_A, PIN_B>,
}

// impl AsPins for PlatformManager<'_> {
//     fn as_pins<'a>(&'a mut self) -> consumer::Pins<'a, PIN_A, PIN_B> {
//         todo!()
//     }
// }

fn main() {
    let mut p = unsafe { Peripherals::steal() };
    {
        let mut output = Output::new(&mut p.PIN_A);
        output.set_high().unwrap();

        let input = Input::new(&mut p.PIN_A);
        drop(input);
    }

    println!("Hello, world!");
}
