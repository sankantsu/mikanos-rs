#!/bin/sh

set -e

usage() {
  echo "Usage: $0 [options]"
  echo ""
  echo "Options:"
  echo "  --wait-debugger    Stop execution at start and wait for a debugger connection (adds -s -S to QEMU)"
  exit 1
}

WAIT_DEBUGGER=0
while [ $# -gt 0 ]; do
  case "$1" in
    --wait-debugger)
      WAIT_DEBUGGER=1
      shift
      ;;
    *)
      usage
      ;;
  esac
done

QEMU_OPTIONS=""
if [ "$WAIT_DEBUGGER" -eq 1 ]; then
  QEMU_OPTIONS="-s -S"
fi

# Build bootloader
pushd mikanos-rs-loader && cargo build
popd

# Build kernel
pushd mikanos-rs-kernel && cargo build
popd

# Make EFI system partition
mkdir -p esp/efi/boot
cp target/x86_64-unknown-uefi/debug/mikanos-rs-loader.efi esp/efi/boot/bootx64.efi
cp target/x86_64-unknown-none/debug/mikanos-rs-kernel esp/kernel.elf

# Launch VM
qemu-system-x86_64 \
  -m 1G \
  -serial stdio \
  -drive if=pflash,format=raw,readonly=on,file=assets/OVMF_CODE.fd \
  -drive if=pflash,format=raw,readonly=on,file=assets/OVMF_VARS.fd \
  -drive format=raw,file=fat:rw:esp \
  -device nec-usb-xhci,id=xhci \
  -device usb-mouse \
  -device usb-kbd \
  $QEMU_OPTIONS
