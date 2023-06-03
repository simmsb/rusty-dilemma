flash:
  cargo objcopy --release --no-default-features -- target/binary.elf
  picotool load -f ./target/binary.elf
  picotool reboot

flash-bl:
  nix build .#bl-binaries
  picotool load ./result/boot.elf
  picotool load ./result/binary.elf
  picotool reboot

dbg-left:
  cargo objcopy --no-default-features -- target/binary.elf
  probe-rs-cli run --probe cafe:4005:6E16C4033956C9E2 --chip RP2040 target/binary.elf --speed 400

dbg-right:
  nix build .#debug-binaries
  probe-rs-cli run --probe cafe:4005:6E16C40339A0F7B2 --chip RP2040 ./result/binary.elf --speed 400
