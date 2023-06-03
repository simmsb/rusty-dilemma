# Rust firmware for the dilemma v2

## Building

- `cargo objcopy --release -- target/binary.elf`
- `picotool load -f ./target/binary.elf`
- `picotool reboot`

(You can use either the nix flake or install picotool and cargo-binutils)
