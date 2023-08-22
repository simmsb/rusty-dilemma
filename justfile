flash:
  cargo objcopy --release -- target/binary.elf
  until picotool load -f ./target/binary.elf; do echo "trying again"; sleep 1; done
  picotool reboot

flash-bl:
  nix build .#bl-binaries
  picotool load ./result/boot.elf
  picotool load ./result/binary.elf
  picotool reboot

dbg-left:
  cargo objcopy --no-default-features --features probe -- target/binary.elf
  probe-rs-cli run --probe cafe:4005:6E16C4033956C9E2 --chip RP2040 target/binary.elf --speed 400
