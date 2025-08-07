#!/usr/bin/python3

from sys import argv

from pyronic.client import *

ipc = IPCClient()

handle1 = ipc.alloc_raw(0x40, 0xfff00100)
buf1 = handle1.read()
print("====== PPC memory ======")
hexdump(buf1)

buf2 = bytearray()
handle2 = ipc.alloc_raw(0x40, 0x0d806840)
for i in range(16):
    buf2 += handle2.read32(i * 4)

print("====== EXI memory ======")
hexdump(buf2)
