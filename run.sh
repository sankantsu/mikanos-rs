#!/bin/sh

set -ex

# Build bootloader
pushd mikanos-rs-loader && cargo build
popd

# Build kernel
pushd mikanos-rs-kernel && cargo build
popd

# Make EFI system partition
mkdir -p esp/efi/boot
cp target/x86_64-unknown-uefi/debug/mikanos-rs-loader.efi esp/efi/boot/bootx64.efi
cp target/x86_64-mikanos_rs/debug/mikanos-rs-kernel esp/kernel.elf

# Launch VM
qemu-system-x86_64 \
  -monitor stdio \
  -drive if=pflash,format=raw,readonly=on,file=assets/OVMF_CODE.fd \
  -drive if=pflash,format=raw,readonly=on,file=assets/OVMF_VARS.fd \
  -drive format=raw,file=fat:rw:esp
