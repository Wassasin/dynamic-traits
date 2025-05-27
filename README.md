# Dynamic HAL traits
Exploration into using `dyn` to minimize code size in Rust `no_std` embedded contexts, ideally without requiring `alloc`.

## Structure
This example is split into 4 parts:
* `hal`: a mock HAL library based on Embassy
* `traits`: generic embedded-hal and friends conversion traits
* `consumer`: a high level library crate that abuses bit-banging pins and communication peripherals
* `bin/main`: an example application that makes use of this all

## Problem statement

### Firmware binaries for multiple SKU's or revisions
Imagine that you have an embedded firmware project that you want to deploy across multiple hardware products. Say you have a single design, but you have a *light* and a *deluxe* variant, for which a different set of parts of the board are populated. Or if you have a longer ongoing product line, you have multiple revisions of the same design.

Each of these variants use the same microcontroller, but the pinout of the microcontroller differs slightly. Secondly, you want to change the configuration of these pins *at runtime*.

From a software management perspective, it would be ideal if you could push a single firmware binary image to all these products. During boot, the software would detect which variant of the product it is running on, and configure everything appropriately.

### Generic business logic
This product has a service running that needs to communicate to a second chipset. This service is implemented by a second team, and is generic over the actual hardware that it is running on. Hence everything is implemented in terms of `embedded-hal` traits and similar.

A complicating factor for this is that the communication interface between the two chipsets has some *quirks*. Sometimes you just need to fiddle a pin belonging to I2C a few times to get the other device in the correct mode. This typically happens after reinitialization. In another scenario the digital interface can switch between SPI and UART. The chip boots in UART mode, but after sending a sequence of bytes it changes over to SPI. 

In both cases you need to switch between a communication interface driver and bit banging the individual digital lines in this generic business logic. There are currently no standardized traits to configure/deconfigure pins and interface in Rust to help you, but you can make your own.

### Generics vs Type erasure
Rust as a language nudges the user to use [generics](https://doc.rust-lang.org/rust-by-example/generics.html) in their code. This is great for creating readable code and prevents code duplication. Generics come with the downside that the compiler tends to duplicate generated code for each instantiation of the generics.

In normal applications this may or may not be an issue. In embedded contexts where non-volatile memory is scarce this is certainly an issue (if you instantiate generics code more than once).

As an illustration of this issue, first generation Rust Embedded HAL implementations tended to use separate types for each peripheral (so also each individual pin), everywhere. To fix this issue, for some types where this makes sense, a generic `Any` version of the driver was added to which can be [degraded](https://github.com/nrf-rs/nrf-hal/blob/dcf6c1ee4680fd6c130703f24fb89915412b7e94/nrf-hal-common/src/gpio.rs#L57-L63). These types would have the underlying peripheral instances erased. In the case of [embassy HAL](https://docs.embassy.dev/embassy-nrf/git/nrf52840/gpio/struct.Input.html#method.new) constructing a driver always yields a degraded `Any` instance. In this respect we have lost the 'zero-cost abstraction' that Rust is known for thanks to the added pin number or register address as a member variable at runtime, in exchange for better ergonomics and smaller generated code size.

As seen in the [embassy GPIO implementation](https://docs.embassy.dev/embassy-nrf/git/nrf52840/gpio/struct.Input.html#method.new) this type erasure only holds for after construction. The construction itself is still parameterized for a concrete peripheral instance type. Hence we generate code for each construction separately. This is for a very good reason: initializing a peripheral might actually be significantly different depending on the exact instance of the peripheral. 

For typical embedded applications this is no problem at all: driver initialization is typically the least amount of the generated code. However in the above scenario involving multiple hardware SKUs, the same firmware binary, and generic business logic, this construction results in vastly duplicated code generation.

## Solution
In this example crate you will find the generic business logic in `src/consumer.rs`. The application code lives in `bin/main.rs`.

First we introduce traits to convert anything into digital logic pin drivers or communication interfaces (i.e. from anything to something that implements `embedded-hal` traits):

```rust
pub trait AsOutput {
    type Target: OutputPin;
    fn as_output(self) -> Self::Target;
}

pub trait AsIoReadWriteDevice {
    type Target: Read + Write;
    fn as_io_read_write(self) -> Self::Target;
}
```

However, we do not want to reconfigure unitary things, we something have to take multiple peripherals to initialize say an UART driver.

```rust
impl<'a> Uart<'a> {
    pub fn new<T: Instance>(
        peri: Peri<'a, T>,
        rx: Peri<'a, impl RxPin<T>>,
        tx: Peri<'a, impl TxPin<T>>,
    ) -> Self {
        ...
    }
}
```

Instead of providing maximum flexibility, we say that given a board in the Board Support Package (BSP) sense can be put into a *mode*, and you can swap between modes assuming that you have dropped everything related to the mode. For this we take a `&mut self` reference, and the thing that we build keeps that lifetime around. When the mode is dropped, the mutable reference gets dropped too, and we can configure a new mode. In our example we have two modes, configure as pins (for bitbanging) and configure as uart:

```rust
pub trait AsPinsMut {
    type RX<'a>: AsOutput + AsInput
    where
        Self: 'a;
    type TX<'a>: AsOutput + AsInput
    where
        Self: 'a;
    fn as_pins(&mut self) -> Pins<Self::RX<'_>, Self::TX<'_>>;
}

pub trait AsUartMut {
    type Target<'a>: AsIoReadWriteDevice
    where
        Self: 'a;
    fn as_uart(&mut self) -> Self::Target<'_>;
}
```

This is then used as follows:

```rust
match state {
    FeatureState::PowerOn => {
        let pins = board.as_pins();

        // Weird chip on the other side needs the bus "de-gaussed"
        let mut rx_pin = AsOutput::as_output(pins.rx);
        let mut tx_pin = AsOutput::as_output(pins.tx);

        rx_pin.set_high().unwrap();
        tx_pin.set_high().unwrap();

        state = FeatureState::FullBus;
    }
    FeatureState::FullBus => {
        let uart_bus = board.as_uart();
        ...
    }
}
```

In our example application we have a single microcontroller with four pins and three uart peripheral instances. Now we want to map these pins to a board configuration depending on the board variant we detect at runtime. 

```rust
struct $board<'a> {
    pins: consumer::Pins<
        PinWrapper<'a, dynamic_traits::hal::peripherals::$pin_rx>,
        PinWrapper<'a, dynamic_traits::hal::peripherals::$pin_tx>,
    >,
    uart: Peri<'a, dynamic_traits::hal::peripherals::$uart>,
}
```

```rust
match board {
    Boards::A => &mut BoardA {
        pins: Pins {
            rx: PinWrapper(p.PIN_A.reborrow()),
            tx: PinWrapper(p.PIN_B.reborrow()),
        },
        uart: p.UART0.reborrow(),
    },
    ...
};
```

Secondly given this board configuration we want to implement conversion into these modes. For this we leverage the [lifetime splitting feature](https://github.com/embassy-rs/embassy/blob/1d8e4fd970ea1794e5a726781442f13c2b2c2b66/embassy-hal-internal/src/peripheral.rs#L58-L62) in embassy.

```rust
impl AsUartMut for $board<'_> {
    type Target<'a>
        = Uart<'a>
    where
        Self: 'a;

    fn as_uart(&mut self) -> Self::Target<'_> {
        Uart::new(
            self.uart.reborrow(),
            self.pins.rx.0.reborrow(),
            self.pins.tx.0.reborrow(),
        )
    }
}
```

We now could call the `consumer` library crate for each of these devices as follows:

```rust
match board {
    Boards::A => {
        let board = BoardA {
            pins: Pins {
                rx: PinWrapper(p.PIN_A.reborrow()),
                tx: PinWrapper(p.PIN_B.reborrow()),
            },
            uart: p.UART0.reborrow(),
        };
        consumer::run(board).await;
    },
    Boards::B => {
        ...
    },
```

But that would potentially result in a separate block of code being generated for each variant, which we would like to avoid. A thought would be to use `&dyn` trait objects to solve this, by defining a super trait that depends on all the mode traits, and casting references to our board to a `&dyn mut` of that supertrait. These `dyn` references are in fact a vtable construct that for each trait method refer to a function pointer, depending on the type. This would result in our code not being duplicated anymore! If we try this:

```rust
trait GenericBoard: AsPinsMut + AsUartMut {}
```

```rust
let board: &mut dyn GenericBoard = match board { ... };
```

This unfortunately results in:

```
error[E0038]: the trait `AsUartMut` is not dyn compatible
   --> src/bin/main.rs:164:33
    |
164 |             let board: &mut dyn GenericBoard = match board {
    |                                 ^^^^^^^^^^^^ `AsUartMut` is not dyn compatible
```

We can only build this vtable when it makes sense. Under the Rust type system, [dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility) defines when this makes sense, and it is basically a long list of requirements that your trait needs to meet.

In our particular case, `AsUartMut` has a associated type `Target<'a>`, for which the lifetime depends on the callsite of the conversion/construction method, as well as the target type itself which depends on the Uart driver that happens to be associated with your HAL implementation.

My initial solve for this is to roll my own vtable implementation to get around these constraints. I was also sure that these constraints could be satisfied for our application in concept, but was not sure how to express it yet in the type system.

After this solution I found in fact that is also OK in this case to use `dyn`.

We know it is OK in our usecase because:
* the application instantiates the same drivers (with the same type) for each board variant, just the construction is different.
* We only pass the lifetime in the associated type, because we have to be generic over this type in our generic business logic. If we don't have an associated type (because the above point holds), the lifetime is contained to the callsite and has no impact on dyn compatibility.

So in *our application* we can define a dyn compatible trait for exactly our used modes and their driver types:

```rust
trait DynBoard {
    fn as_pins_compat(&mut self) -> Pins<&mut dyn DynPin, &mut dyn DynPin>;
    fn as_uart_compat(&mut self) -> Uart<'_>;
}
```

As far as I know, we cannot implement this trait for each type that implements the subtraits like so:

```rust
impl<T> DynBoard for T
where
    T:
        for<'a> AsPinsMut<
            RX<'a> = &'a mut dyn DynPin,
            TX<'a> = &'a mut dyn DynPin
        > +
        for<'a> AsUartMut<Target<'a> = Uart<'a>>,
{
    fn as_pins_compat(&mut self) -> Pins<&mut dyn DynPin, &mut dyn DynPin> {
        AsPinsMut::as_pins(self)
    }

    fn as_uart_compat(&mut self) -> Uart<'_> {
        AsUartMut::as_uart(self)
    }
}
```

Specifically, I am not sure how to constraint type `T` such that it is implemented for any type that implements the mode traits for which the targets are the driver implementations we use.

However we are application-level now, so we can implement it for each of our board types using macro's without too much difficulty:

```rust
impl DynBoard for $board<'_> {
    fn as_pins_compat(&mut self) -> Pins<&'_ mut dyn DynPin, &'_ mut dyn DynPin> {
        AsPinsMut::as_pins(self)
    }

    fn as_uart_compat(&mut self) -> Uart<'_> {
        AsUartMut::as_uart(self)
    }
}
```

And coerce a `&mut` for each of our boards into `&mut dyn`:

```rust
let board: &mut dyn DynBoard = match board { ... }
```

This reference can not be used to pass it to `consumer::run`, until we implement our mode traits for `&mut dyn` of this trait:

```rust
impl AsPinsMut for &mut dyn DynBoard {
    type RX<'a>
        = &'a mut dyn DynPin
    where
        Self: 'a;

    type TX<'a>
        = &'a mut dyn DynPin
    where
        Self: 'a;

    fn as_pins(&mut self) -> Pins<Self::RX<'_>, Self::TX<'_>> {
        self.as_pins_compat()
    }
}

impl AsUartMut for &mut dyn DynBoard {
    type Target<'a>
        = Uart<'a>
    where
        Self: 'a;

    fn as_uart(&mut self) -> Self::Target<'_> {
        self.as_uart_compat()
    }
}
```

Which works! We can now call:

```rust
let board: &mut dyn DynBoard = match board { ... }
consumer::run(board).await;
```

## TODO
- [ ] PinWrapper must be owned by the board, but perhaps we can remove it
- [ ] In main() the match blob needs to be split out
- [ ] Can we generate the macro in a nice way for each board?