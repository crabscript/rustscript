#!/bin/bash

# Check if Rust is installed by checking cargo version
if cargo --version &>/dev/null; then
  echo "Rust is installed."

  # Commands to build and move executables to a bin directory
  echo "Building executables..."
  cargo build --release
  mkdir -p bin
  mv ./target/release/oxidate bin/
  mv ./target/release/ignite bin/
  echo "Build complete. Executables are in the bin directory."

  echo "Adding temporary aliases for executables..."
  CWD=$(pwd)
  alias oxidate="$CWD/bin/oxidate"
  alias ignite="$CWD/bin/ignite"

  echo "To use the executables, run the following commands:"
  echo "oxidate --help"
  echo "ignite --help"

else
  echo "Rust is not installed. Please install Rust to proceed."
  exit 1
fi
