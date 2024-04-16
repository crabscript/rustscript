#!/bin/bash

# Check if Rust is installed by checking cargo version
if cargo --version &>/dev/null; then
  echo "Rust is installed."
  # Commands to build and move executables to a bin directory
  cargo build --release
  mkdir -p bin
  mv ./target/release/oxidate bin/
  mv ./target/release/ignite bin/
else
  echo "Rust is not installed. Please install Rust to proceed."
  exit 1
fi
