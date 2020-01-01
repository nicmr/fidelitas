#!/bin/bash

for lib in crt2.o dllcrt2.o libmsvcrt.a;
do cp -v /usr/x86_64-w64-mingw32/lib/$lib /usr/local/rustup/toolchains/*-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-pc-windows-gnu/lib/;
done