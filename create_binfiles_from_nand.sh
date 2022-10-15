#!/bin/sh
# nand.bin is the raw nand dump consisting of 
#  - 4096 blocks each containing 64 pages
#   - each page is 2112 bytes (2048 data + 64 ECC)
# plus a 1024 byte footer called keys.bin which contains
#  - 256 byte header containing an ACSII string padded to the end with null bytes
#  - 128 bytes of OTP data
#  - 128 bytes of useless padding
#  - 256 bytes of SEEPROM data
#  - 256 bytes of useless padding
dd if=nand.bin of=keys.bin bs=1 count=1024 skip=553648128
dd if=keys.bin of=otp.bin bs=1 count=128 skip=256
dd if=keys.bin of=seeprom.bin bs=1 count=256 skip=512
# Print the header from keys.bin
head -c 256 keys.bin
