#!/bin/bash

set -euo pipefail

(
cd "$(dirname "$0")"

g++ -c -O2 \
	Window.cpp \
	WindowFFI.cpp

ar rcs ../../deps/libwindow.a Window.o WindowFFI.o

rm Window.o WindowFFI.o
)
