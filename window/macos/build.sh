#!/bin/bash

set -euo pipefail

(
cd "$(dirname "$0")"

swiftc -c -O \
	-parse-as-library \
	-import-objc-header window.h \
	Window.swift \
	WindowFFI.swift

ar rcs ../../deps/libwindow.a Window.o WindowFFI.o

rm Window.o WindowFFI.o
)
