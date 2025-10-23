# oxython

oxython is the Python programming language implemented in Rust.

![CI](https://github.com/gdonald/oxython/workflows/CI/badge.svg) [![codecov](https://codecov.io/gh/gdonald/oxython/graph/badge.svg?token=GQ4LA1VMRE)](https://codecov.io/gh/gdonald/oxython)

---

## Development Roadmap

- [x] Core Foundation & Lexing
    - [x] Project Setup: Initialize the Rust project with Cargo and add dependencies like `inkwell` (for JIT) and any chosen parsing helpers.
    - [x] Object/Value Type: Define the central `Object` struct/enum to wrap all primitive data (`i64`, `f64`, `String`) and include a simple reference counter.
    - [x] Token Definition: Define the core `Token` enum, including types for numbers, floats, strings, basic math, and keywords (`if`, `fn`, `class`).
    - [x] Lexer Implementation: Implement the `Lexer` to convert source code text into a stream of tokens, correctly handling literal formats for integers, floats, and quoted strings.

- [x] Bytecode Virtual Machine (VM)
    - [x] Bytecode Instruction Set: Define a simple set of `OpCode` enums. Start with `LOAD_CONST`, `ADD`, `SUB`, `JUMP`, and stack management instructions.
    - [x] VM Structure: Define the `VM` struct. It needs a main stack (for `Object` values), a program counter (PC), and a mechanism to manage execution context.
    - [x] VM Execution Loop: Implement the core `run()` method—a simple loop that fetches the next `OpCode`, decodes it, executes the action, and increments the PC.
    - [x] Object Operations: Implement the VM logic for instructions like `ADD`. The VM must check operand types and handle the operation since everything is an `Object`.

- [x] Compiler & Bytecode Generation
    - [x] Simple Parser/Compiler: Implement a core module to read tokens and emit the `OpCode` vector (the bytecode), prioritizing simple implementation over a complex AST.
    - [x] Variable Management: Implement a basic symbol table to track local variables and assign them memory slots for use with `LOAD_VAR`/`STORE_VAR` opcodes.
    - [x] Function/Block Compilation: Implement logic to compile code blocks and functions, arranging their instructions sequentially and handling the necessary jump/return logic.
- [ ] First-Class Functions:
        - [x] Syntax & Parsing: Extend the compiler to recognize `def name(args): ...` declarations and emit function objects via a new `MAKE_FUNCTION` opcode.
        - [x] Call Frames: Introduce call stack frames in the VM with instruction pointers, stack bases, and local scopes; add `CALL`/`RETURN` opcodes.
        - [x] Locals & Scope: Support argument binding and function-local variables (global scope first).
        - [x] Testing: Add compiler/VM unit tests covering function definition, invocation, and return values, plus example scripts that print function results.
        - [ ] Closures & Nested Functions (Optional): Capture non-local variables via environment structures and update the compiler/VM so inner functions can reference outer scopes.
    - [x] Initial Execution Test: Connect the whole chain: Lexer $\rightarrow$ Compiler $\rightarrow$ VM. Compile a small script and run the resulting bytecode in the VM to verify functionality.

- [ ] Object-Oriented Features & Types
    - [ ] Class Definition: Implement a `Class` structure in the runtime that stores fields (data attributes) and methods (bytecode functions).
    - [ ] Instance Creation: Implement the bytecode instruction (`NEW_OBJECT`) and VM logic to create an instance of a class, which is an `Object` referencing its class definition.
    - [ ] Attribute Access: Add opcodes for `GET_ATTR`/`SET_ATTR`, instance dictionaries, and class method tables with resolution order.
    - [ ] Method Binding & `self`: Ensure functions defined in class bodies are stored as methods and automatically bind the instance as the first argument during calls.
    - [ ] Method Dispatch: Implement the logic for calling a method on an object (`CALL_METHOD` opcode) using the lookup rules established above.
    - [ ] Construction Flow: Support calling a class to create instances and execute `__init__` on the new object.
    - [ ] Inheritance & Advanced Behavior (Stretch): Implement single inheritance, `super()`, and special methods such as `__str__`/`__iter__`.
    - [ ] Optional Type System: Implement a basic structure that allows the programmer to *annotate* variables or function arguments with types, but allows the compiler to skip strict checking if no type is provided.

- [ ] JIT Compilation with LLVM
    - [ ] LLVM/Inkwell Setup: Set up the LLVM `Context`, `Module`, and `ExecutionEngine` using the `inkwell` crate to manage the JIT compilation process.
    - [ ] Hotspot Identification: Modify the VM to track block execution counts. Flag a bytecode block as "hot" when it is frequently executed.
    - [ ] Bytecode to IR Translator: Implement the core translation function: it takes a sequence of bytecode and generates equivalent, optimized LLVM IR instructions using the `inkwell` builder.
    - [ ] Runtime Switch: Implement the dynamic execution switch. When the VM reaches a JIT-compiled block, it calls the native machine code function generated by LLVM instead of interpreting the bytecode.
