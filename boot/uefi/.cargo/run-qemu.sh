#!/usr/bin/env bash

set -ex

uefi_bin=$1
outdir=$(dirname $uefi_bin)

# Disassamble
llvm-objdump --section-headers --source -d $uefi_bin > $uefi_bin.S

# Copy startup script
cp .cargo/startup.nsh $outdir/

# Launch qemu
qemu=qemu-system-aarch64
bios=.cargo/QEMU_EFI.fd
machine_args="-M virt -cpu cortex-a72 -smp 1 -m 1G"
# $qemu -M virt,dumpdtb=$outdir/device-tree.dtb -cpu cortex-a72 -smp 1 -m 1G
$qemu $machine_args -bios $bios -drive index=0,format=raw,file=fat:rw:$outdir -net none -monitor none -nographic -serial stdio


# Launch qemu
# qemu=qemu-system-x86_64
# bios=.cargo/OVMF.fd
# machine_args="-cpu qemu64"
# $qemu $machine_args -bios $bios -drive file=$tmp_img,index=0,media=disk,format=raw -net none -monitor none -nographic -serial stdio