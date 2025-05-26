# Dynamic HAL traits
This example is split into 4 parts:
* `hal`: a mock HAL library based on Embassy
* `traits`: generic embedded-hal and friends conversion traits
* `consumer`: a high level library crate that abuses bit-banging pins and communication peripherals
* `bin/main`: an example application that makes use of this all

## TODO
- [ ] PinWrapper must be owned by the board, but perhaps we can remove it
- [ ] In main() the match blob needs to be split out
- [ ] Can we generate the macro in a nice way for each board?