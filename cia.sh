#!/bin/bash
# builds the .cia file for grairc

if ! command -v bannertool &> /dev/null
then
    echo "bannertool not found, please check your PATH or install it."
    exit 1
fi

if ! command -v makerom &> /dev/null
then
    echo "makerom not found, please check your PATH or install it."
    exit 1
fi

cargo 3ds build --release

bannertool makebanner -o target/armv6k-nintendo-3ds/release/banner.bnr \
    -i metadata/banner.png \
    -a metadata/banner.wav

makerom -f cia -target t -exefslogo -o target/armv6k-nintendo-3ds/release/grairc.cia \
    -icon target/armv6k-nintendo-3ds/release/grairc.smdh \
    -banner target/armv6k-nintendo-3ds/release/banner.bnr \
    -elf target/armv6k-nintendo-3ds/release/grairc.elf \
    -rsf metadata/grairc.rsf

echo "outputted banner file to target/armv6k-nintendo-3ds/release/banner.bnr"
echo "outputted cia file to target/armv6k-nintendo-3ds/release/grairc.cia"