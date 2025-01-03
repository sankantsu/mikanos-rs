#!/bin/sh

set -ex

# Build EFI image
cargo build --target x86_64-unknown-uefi

# Make EFI system partition
mkdir -p esp/efi/boot
cp target/x86_64-unknown-uefi/debug/mikanos-rs-loader.efi esp/efi/boot/bootx64.efi

# Launch VM
qemu-system-x86_64 \
  -drive if=pflash,format=raw,readonly=on,file=assets/OVMF_CODE.fd \
  -drive if=pflash,format=raw,readonly=on,file=assets/OVMF_VARS.fd \
  -drive format=raw,file=fat:rw:esp
