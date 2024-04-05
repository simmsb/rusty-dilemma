flash:
  cargo strip --bin binary --release -Zbuild-std=core,alloc,panic_abort -Zbuild-std-features=panic_immediate_abort -- --strip-all -o target/binary.elf
  echo "Binary size is $(ls -lah target/binary.elf)"
  until picotool load -f ./target/binary.elf; do echo "trying again"; sleep 1; done
  picotool reboot

flash-bl:
  cargo build --release -Zbuild-std=core,alloc,panic_abort -Zbuild-std-features=panic_immediate_abort
  cp ./target/thumbv6m-none-eabi/release/boot ./target/boot.elf
  until picotool load -f ./target/boot.elf; do echo "trying again"; sleep 1; done

dbg-left:
  cargo objcopy --no-default-features --features probe -- target/binary.elf
  probe-rs-cli run --probe cafe:4005:6E16C4033956C9E2 --chip RP2040 target/binary.elf --speed 400
