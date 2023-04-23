flash:
  nix build .#binaries
  picotool load ./result/left.elf
  picotool reboot

flash-bl:
  nix build .#bl-binaries
  picotool load ./result/boot.elf
  picotool load ./result/left.elf
  picotool reboot
