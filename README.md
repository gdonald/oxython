## oxython

oxython is the Python programming language implemented in Rust.

⚠️ &nbsp;It's still very early in development.

🙂 &nbsp;[PRs](https://github.com/gdonald/oxython/pulls) and [new](https://github.com/gdonald/oxython/issues/new) [issues](https://github.com/gdonald/oxython/issues) are welcome.

###

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/gdonald/oxython/blob/main/LICENSE) [![CI](https://github.com/gdonald/oxython/workflows/CI/badge.svg)](https://github.com/gdonald/oxython/actions) [![codecov](https://codecov.io/gh/gdonald/oxython/graph/badge.svg?token=GQ4LA1VMRE)](https://codecov.io/gh/gdonald/oxython)

### Run with cargo

```bash
# Run the REPL:
cargo run

# Run a script:
cargo run -- examples/oop/class.py
```

### Run tests

```bash
cargo test

# code coverage
cargo tarpaulin --out Stdout

# or
cargo tarpaulin --out Html
```

### Architecture

![Architecture](https://raw.githubusercontent.com/gdonald/oxython/refs/heads/main/dia/architecture_notes.png)

![Architecture](https://raw.githubusercontent.com/gdonald/oxython/refs/heads/main/dia/architecture.png)

### Development Roadmap

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

- [x] First-Class Functions:
    - [x] Syntax & Parsing: Extend the compiler to recognize `def name(args): ...` declarations and emit function objects via a new `MAKE_FUNCTION` opcode.
    - [x] Call Frames: Introduce call stack frames in the VM with instruction pointers, stack bases, and local scopes; add `CALL`/`RETURN` opcodes.
    - [x] Locals & Scope: Support argument binding and function-local variables (global scope first).
    - [x] Testing: Add compiler/VM unit tests covering function definition, invocation, and return values, plus example scripts that print function results.
    - [x] Closures & Nested Functions: Capture non-local variables via environment structures and update the compiler/VM so inner functions can reference outer scopes.
    - [x] `nonlocal` Assignments: Expand semantics so nested functions can rebind variables declared in outer scopes once the keyword is available.
    - [x] Initial Execution Test: Connect the whole chain: Lexer $\rightarrow$ Compiler $\rightarrow$ VM. Compile a small script and run the resulting bytecode in the VM to verify functionality.

- [x] Object-Oriented Features
    - [x] Foundation: Add `class` keyword token, `ClassObject` and `InstanceObject` types to object model, and class-related opcodes (`OpMakeClass`, `OpGetAttr`, `OpSetAttr`).
    - [x] Class Parsing: Implement `parse_class_statement` in compiler to recognize `class ClassName:` syntax, parse method definitions within class body, and emit `OpMakeClass` instruction.
    - [x] Compiler: General Attribute Access
        - [x] Implement hybrid approach: hardcoded opcodes for built-in methods (`.append()`, `.lower()`) and general `OpGetAttr` for user-defined class attributes.
        - [x] Distinguish between attribute access (reading) and method calls (with parentheses) to properly handle both cases.
        - [x] Implement attribute assignment detection: recognize `obj.attr = value` pattern and emit `OpSetAttr` instead of treating it as a regular assignment.
    - [x] VM: Class Creation (`OpMakeClass`)
        - [x] Pop method count from instruction stream, then pop that many method name/function pairs from stack.
        - [x] Create `ClassObject` with class name and methods stored in HashMap.
        - [x] Push class object onto stack for subsequent `OpDefineGlobal`.
    - [x] VM: Instance Creation
        - [x] Modify `call_value` to detect when a `Class` object is being called (not a function).
        - [x] Create new `InstanceObject` with reference to the class.
        - [x] Look up `__init__` method in class methods; if found, call it with the instance as first argument (`self`) plus any constructor arguments.
        - [x] Push the instance onto the stack as the result.
    - [x] VM: Attribute Access (`OpGetAttr`)
        - [x] Pop attribute name (string constant) and object from stack.
        - [x] If object is `Instance`: first check instance fields, then check class methods.
        - [x] If attribute is a method from the class, create a bound method (wrapper that pre-fills `self` as first argument).
        - [x] Push the attribute value or bound method onto stack.
        - [x] Handle errors for missing attributes gracefully.
    - [x] VM: Attribute Assignment (`OpSetAttr`)
        - [x] Pop value, attribute name, and instance from stack.
        - [x] Call `instance.set_field(name, value)` to update or add the field.
        - [x] Since instances use `Rc<RefCell<InstanceObject>>`, use `.borrow_mut()` to modify in place.
        - [x] Push the instance back onto stack (or handle as mutation).
    - [x] VM: Method Binding & `self`
        - [x] Create a `BoundMethod` wrapper type that stores both the instance and the method function.
        - [x] When `OpCall` encounters a bound method, automatically inject the instance as the first argument before the user-provided arguments.
        - [x] Ensure arity checking accounts for the implicit `self` parameter.
    - [x] Testing: Basic Class Functionality
        - [x] Enable `examples/oop/class.py` test in `examples_runner.rs`.
        - [x] Verify class definition, instance creation, attribute assignment (`self.name = name`), attribute access, and method calls work correctly.
        - [x] Add unit tests for class creation, instance creation, `__init__` execution, attribute get/set, and method dispatch.
    - [x] Inheritance & Advanced Behavior
        - [x] Single inheritance with parent class reference in `ClassObject`.
        - [x] Method resolution order (MRO) for attribute lookup through parent chain.
        - [x] `super()` builtin for calling parent methods.
        - [x] Special methods: `__str__`, `__repr__`, `__iter__`, `__next__`, etc.

- [ ] Optional Type System
    - [x] Foundation: Type Representation
        - [x] Define `Type` enum in object model to represent basic types (`int`, `float`, `str`, `bool`, `list`, `dict`, class types).
        - [x] Add optional `type_annotation` field to variable slots and function parameters.
        - [x] Extend token definitions to include colon (`:`) for type annotations and arrow (`->`) for return types.
    - [x] Lexer & Parser Extensions
        - [x] Extend lexer to recognize type annotation syntax (e.g., `x: int`, `def func(a: str) -> int:`).
        - [x] Modify parser to parse variable annotations: `name: type = value`.
        - [x] Modify parser to parse function parameter annotations: `def func(param: type):`.
        - [x] Modify parser to parse function return type annotations: `def func() -> type:`.
        - [x] Store type annotations in AST/compiler metadata without enforcing them.
    - [ ] Runtime Type Information
        - [x] Function Introspection Attributes (Phase 1: Basic Attributes)
            - [x] Extend `OpGetAttr` in VM to support `Function` and `FunctionPrototype` objects.
            - [x] Implement `__name__` attribute access for functions (returns function name as string).
            - [x] Add `__doc__` field to `FunctionObject` and `FunctionPrototype` for docstrings.
            - [x] Implement `__doc__` attribute access for functions.
            - [x] Create example demonstrating basic function attribute access (`func.__name__`, `func.__doc__`).
            - [x] Write tests for basic function attributes.
        - [x] Function Introspection Attributes (Phase 2: Type Annotations)
            - [x] Implement `__annotations__` attribute that returns a Dict object with parameter names and type annotations.
            - [x] Format `__annotations__` dict: parameter names as keys, type names as string values, 'return' key for return type.
            - [x] Remove `#[allow(dead_code)]` from `parameter_types` and `return_type` fields once they're actively used.
            - [x] Create example demonstrating `__annotations__` access and type introspection.
            - [x] Write tests for `__annotations__` attribute.
        - [x] Function Introspection Attributes (Phase 3: Code & Module)
            - [x] Implement `__code__` attribute (returns reference to the function's bytecode chunk or code object).
            - [x] Add `module` field to `FunctionObject` and `FunctionPrototype` to track defining module.
            - [x] Implement `__module__` attribute access (returns module name as string, default to `<script>`).
            - [x] Create example demonstrating `__code__` and `__module__` access.
            - [x] Write tests for `__code__` and `__module__` attributes.
        - [x] Function Introspection Attributes (Phase 4: Namespaces & Closures)
            - [x] Add `globals` field to `FunctionObject` to reference the global namespace where function was defined.
            - [x] Implement `__globals__` attribute (returns reference to global namespace dict).
            - [x] Implement `__closure__` attribute (returns tuple of cell objects from upvalues, or None).
            - [x] Implement `__qualname__` attribute for qualified names (handles nested functions).
            - [x] Create example demonstrating namespace and closure introspection.
            - [x] Write tests for `__globals__`, `__closure__`, and `__qualname__`.
        - [x] Function Introspection Attributes (Phase 5: Default Parameters)
            - [x] Note: Default Parameters are now implemented (see "Advanced Function Features").
            - [x] Implement `__defaults__` attribute (tuple of default values for positional parameters).
            - [x] Create example demonstrating default parameter introspection.
            - [x] Write tests for default parameter attributes.
        - [x] Store variable type annotations in symbol table (compiler-time only, no runtime enforcement).
        - [ ] Implement `type()` builtin to query object types at runtime.
    - [ ] Optional Type Checking (Compiler-Time)
        - [ ] Add compiler flag/mode to enable optional type checking.
        - [ ] Implement basic type checker that validates annotated variables and function calls.
        - [ ] Report type mismatches as warnings or errors (configurable).
        - [ ] Allow unannotated code to bypass type checking entirely.
    - [ ] Testing & Examples
        - [ ] Create examples demonstrating type annotations without enforcement.
        - [ ] Create examples with type checking enabled showing caught type errors.
        - [ ] Add unit tests for type annotation parsing and storage.
        - [ ] Add integration tests for optional type checking mode.

- [ ] Module System
    - [ ] Module Object: Define a `Module` runtime type that contains its own global namespace (symbol table) and metadata (name, file path).
    - [ ] Import Syntax & Parsing: Extend the parser to recognize `import module`, `from module import name`, and `import module as alias` statements.
    - [ ] File Loading: Implement the file system layer to locate and read `.py` files from disk based on module names and search paths (current directory, then standard library paths).
    - [ ] Module Cache: Create a module registry/cache in the VM to store already-loaded modules by their fully qualified name, preventing redundant compilation and execution.
    - [ ] Compilation & Initialization: When a module is imported, lex and compile its source into bytecode, then execute that bytecode in a fresh namespace to populate the module's globals.
    - [ ] Import Opcodes: Add bytecode instructions (`IMPORT_MODULE`, `IMPORT_FROM`) so the compiler can emit import operations that the VM will execute at runtime.
    - [ ] Namespace Binding: Implement the logic to bind imported names into the importing module's namespace (e.g., `import math` creates a local binding to the `math` module object).
    - [ ] Attribute Access on Modules: Extend `GET_ATTR` to work on module objects, allowing code like `math.sqrt` to retrieve symbols from a module's namespace.
    - [ ] Circular Import Detection: Add safeguards to detect and handle circular dependencies gracefully (e.g., by marking modules as "loading" and preventing infinite loops).
    - [ ] Relative Imports (Optional): Support relative import syntax (`from . import sibling`, `from .. import parent`) for package-aware module resolution.
    - [ ] Package Support (Optional): Implement package semantics with `__init__.py` files and hierarchical module namespaces (e.g., `package.submodule`).

- [ ] Advanced Function Features (Future Enhancements)
    - [x] Default Parameters: Support optional parameters with default values (e.g., `def func(a, b=10):`).
    - [ ] Variable-Length Arguments: Implement `*args` to accept variable number of positional arguments.
    - [ ] Keyword Arguments: Support `**kwargs` and named argument passing in function calls.
    - [ ] Lambda Expressions: Add support for anonymous functions with `lambda` syntax.
    - [ ] Decorators: Implement decorator syntax (`@decorator`) for function wrapping and metaprogramming.
    - [ ] Generator Functions: Add `yield` keyword and generator protocol for lazy iteration.
    - [ ] Function Introspection: Expose function metadata (`__name__`, `__code__`, `__defaults__`, etc.).

- [ ] JIT Compilation with LLVM
    - [ ] LLVM/Inkwell Setup: Set up the LLVM `Context`, `Module`, and `ExecutionEngine` using the `inkwell` crate to manage the JIT compilation process.
    - [ ] Hotspot Identification: Modify the VM to track block execution counts. Flag a bytecode block as "hot" when it is frequently executed.
    - [ ] Bytecode to IR Translator: Implement the core translation function: it takes a sequence of bytecode and generates equivalent, optimized LLVM IR instructions using the `inkwell` builder.
    - [ ] Runtime Switch: Implement the dynamic execution switch. When the VM reaches a JIT-compiled block, it calls the native machine code function generated by LLVM instead of interpreting the bytecode.

- [ ] Function Introspection Attributes (Phase 5: Default Parameters)
    - [ ] Implement `__kwdefaults__` attribute (dict of default values for keyword-only parameters - Future).
