# Dynamic HAL traits
This example is split into 4 parts:
* `hal`: a mock HAL library based on Embassy
* `traits`: generic embedded-hal and friends conversion traits
* `consumer`: a high level library crate that abuses bit-banging pins and communication peripherals
* `bin/main`: an example application that makes use of this all

## TODO
- [ ] Can we simplify `traits` away?
- [ ] `hal/foreign` is an implementation of the `traits` directly on some peripherals.
- [ ] the original problem statement asks for a abstract/generic way to instantiate the `Platform` struct.
