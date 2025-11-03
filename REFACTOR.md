# VM Module Refactoring

- [x] Phase 1: Extract Opcode Dispatching (~690 lines)
    - [x] Create `src/vm/opcode_dispatcher.rs`
    - [x] Move opcode match statement from `run()` into `dispatch_opcode()`
    - [x] Update `run()` to call `dispatch_opcode()`
    - [x] Add module declaration to `mod.rs`
    - [x] Run `cargo fmt && cargo clippy`
    - [x] Run `cargo test`

- [x] Phase 2: Extract Complex Opcode Handlers
    - [x] 2a: Extract OpMakeFunction (~45 lines)
        - [x] Add `op_make_function()` to `opcodes/functions.rs`
        - [x] Update dispatcher to call new function
        - [x] Run `cargo test`
    - [x] 2b: Extract OpMakeClass (~39 lines)
        - [x] Add `op_make_class()` to `opcodes/classes.rs`
        - [x] Update dispatcher to call new function
        - [x] Run `cargo test`
    - [x] 2c: Extract OpGetAttr (~246 lines)
        - [x] Create `src/vm/opcodes/attributes.rs`
        - [x] Implement `op_get_attr()` with helper functions:
            - [x] `get_instance_attr()`
            - [x] `get_class_attr()`
            - [x] `get_super_proxy_attr()`
            - [x] `get_function_attr()`
            - [x] `get_function_prototype_attr()`
        - [x] Add to `opcodes/mod.rs`
        - [x] Update dispatcher to call new function
        - [x] Run `cargo test`
    - [x] 2d: Extract OpSetAttr (~16 lines)
        - [x] Add `op_set_attr()` to `opcodes/attributes.rs`
        - [x] Update dispatcher to call new function
        - [x] Run `cargo test`
    - [x] 2e: Extract OpInherit (~26 lines)
        - [x] Add `op_inherit()` to `opcodes/classes.rs`
        - [x] Update dispatcher to call new function
        - [x] Run `cargo test`
    - [x] Run `cargo fmt && cargo clippy`

- [x] Phase 3: Extract Function Call Logic (~175 lines)
    - [x] Create `src/vm/function_calls.rs`
    - [x] Move `call_value()` method
    - [x] Move `call_function()` method
    - [x] Add helper methods for call types
    - [x] Add module declaration to `mod.rs`
    - [x] Run `cargo fmt && cargo clippy`
    - [x] Run `cargo test`

- [x] Phase 4: Extract Return Handling (~43 lines)
    - [x] Create `src/vm/return_handler.rs`
    - [x] Move `handle_return()` method
    - [x] Add module declaration to `mod.rs`
    - [x] Run `cargo fmt && cargo clippy`
    - [x] Run `cargo test`

- [x] Phase 5: Simplify Main run() Loop
    - [x] Refactor `run()` to simplified version
    - [x] Adjust dispatcher control flow if needed
    - [x] Run `cargo fmt && cargo clippy`
    - [x] Run `cargo test`

- [ ] Phase 6: Final Verification
    - [ ] Verify `mod.rs` is ~100-150 lines
    - [ ] Verify all tests pass: `cargo test`
    - [ ] Verify no clippy warnings: `cargo clippy`
    - [ ] Verify 100% test coverage: `cargo tarpaulin`
    - [ ] Verify code formatting: `cargo fmt --check`
    - [ ] Update README.md to mark refactoring as complete
