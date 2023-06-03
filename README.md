# Rust firmware for the dilemma v2

https://github.com/simmsb/rusty-dilemma/assets/5330444/2e6345b6-a52b-436a-b9c9-535a1fc93490

## Building

- `cargo objcopy --release -- target/binary.elf`
- `picotool load -f ./target/binary.elf`
- `picotool reboot`

(You can use either the nix flake or install picotool and cargo-binutils)
