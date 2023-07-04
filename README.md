# Rust firmware for the [dilemma v2](https://github.com/Bastardkb/Dilemma)

https://github.com/simmsb/rusty-dilemma/assets/5330444/2e6345b6-a52b-436a-b9c9-535a1fc93490

## Features

- Normal keypresses, mod taps, layers, chords, mouse keys
- Cirque trackpad support, with support for using it to scroll
- Some pretty neopixel animations (that sync between sides, and transition smoothly)
- Single firmware binary, everything works no matter which side is plugged in
- Double tapping the update button puts the mcu into dfu mode
- The device pretends to be a RP Pico and supports being put into DFU mode by
  `picotool`

## Building

- `cargo objcopy --release -- target/binary.elf`
- `picotool load -f ./target/binary.elf`
- `picotool reboot`

(You can use either the nix flake or install picotool and cargo-binutils)
