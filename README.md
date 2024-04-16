# RustScript

## Project Description

RustScript is an innovative programming language that melds the syntactic structure of Rust with the approachability and simplicity of TypeScript. This project aims to create a language that is both familiar to developers and provides a unique development experience. By implementing RustScript using a virtual machine built on Rust, we aspire to delve into the intricacies of programming language implementation, from crafting a language from the ground up to understanding the workings of a virtual machine and bytecode execution.

The RustScript project is not just an academic exercise but a practical exploration into static typing, compilation, and interpretation within the realm of system programming. By deploying RustScript as a standalone executable binary, users will be able to compile RustScript code into `o2` bytecode and execute it on our custom virtual machine. A notable feature of this binary is its strict type checking, ensuring that only well-typed programs proceed to execution.

Our journey is partly inspired by ["Writing An Interpreter In Go,"](https://interpreterbook.com/) which offers insights into language design and interpretation. However, RustScript is our own creation, focusing on the syntax inspired by Rust and TypeScript while forging its own path in programming language design.

## Installation

1. Install Rust on your system: https://www.rust-lang.org/tools/install

```bash
# Run this command to verify installation
cargo --version
```

2. Clone the source code:

```bash
git clone https://github.com/crabscript/rustscript.git
```

3. Build the compiler and virtual machine

```bash
cd rustscript
./build.sh
```

4. The compiler binary is oxidate and the virtual machine is ignite. Both executables are located inside bin directory
5. Run `./bin/oxidate --help` or `./bin/ignite --help` to see the available options
6. You can compile any .rst rustscript code into .o2 bytecode and run it with the ignite virtual machine

```bash
# Assuming you are in the rustscript directory
./bin/oxidate example/hello-world.rst # Should generate example.o2
./bin/ignite hello-world.o2
```

## Project Deliverables

- **Syntax**: RustScript's syntax is a harmonious blend of Rust and TypeScript, offering a familiar yet unique coding experience.
- **Expression-Centric Design**: Every construct in RustScript is an expression, capable of producing a value or a unit (void), ensuring a consistent and predictable programming model.
- **Control Flow**:
  - Conditional statements (`if`, `else`) for branching logic.
  - Loop constructs, including a `for` loop and a Golang-like `while` loop without brackets.
- **Static Typing**: A robust type checking phase to eliminate non well-typed programs before execution, reinforcing code reliability and performance.
- **Data Types**:
  - Primitive types: `int`, `float`, `string`, `bool`, `unit` (void).
- **Functional Features**:
  - Support for higher-order functions, allowing functions to be passed as arguments or assigned to variables.
  - Lambda expressions for concise and flexible function definition.
- **Concurrency**: Implementation of multithreading to leverage modern processor capabilities and enhance performance.

## Reach Goals

- Extend the standard library with a comprehensive set of utilities and functions.
- Advanced types: Arrays (e.g., `T[]`), tuples, and functions, including support for generics in arrays like `int[]`, `float[]`, etc.
- Integrate an interactive RustScript REPL for immediate code evaluation and experimentation.
- Develop a robust ecosystem around RustScript, including package management, tooling, and extensive documentation to foster a community of users and contributors.
- Explore the integration of RustScript in web and network programming, potentially expanding its applicability to broader domains.

RustScript is more than just a programming language; it's a venture into understanding the essence of language design and execution, aiming to provide a powerful tool for developers while offering insights into the complexities of language implementation.
