flash:
  nix build .#binaries
  picotool load -f ./result/right.elf
  picotool reboot

flash-bl:
  nix build .#bl-binaries
  picotool load ./result/boot.elf
  picotool load ./result/left.elf
  picotool reboot

dbg-left:
  nix build .#debug-binaries
  probe-rs-cli run --probe cafe:4005:6E16C4033956C9E2 --chip RP2040 ./result/left.elf --speed 400

dbg-right:
  nix build .#debug-binaries
  probe-rs-cli run --probe cafe:4005:6E16C40339A0F7B2 --chip RP2040 ./result/right.elf --speed 400

