# VM Module Refactoring

- [x] Phase 1: Extract Opcode Dispatching (~690 lines)
    - [x] Create `src/vm/opcode_dispatcher.rs`
    - [x] Move opcode match statement from `run()` into `dispatch_opcode()`
    - [x] Update `run()` to call `dispatch_opcode()`
    - [x] Add module declaration to `mod.rs`
    - [x] Run `cargo fmt && cargo clippy`
    - [x] Run `cargo test`

- [ ] Phase 2: Extract Complex Opcode Handlers
    - [ ] 2a: Extract OpMakeFunction (~45 lines)
        - [ ] Add `op_make_function()` to `opcodes/functions.rs`
        - [ ] Update dispatcher to call new function
        - [ ] Run `cargo test`
    - [ ] 2b: Extract OpMakeClass (~39 lines)
        - [ ] Add `op_make_class()` to `opcodes/classes.rs`
        - [ ] Update dispatcher to call new function
        - [ ] Run `cargo test`
    - [ ] 2c: Extract OpGetAttr (~246 lines)
        - [ ] Create `src/vm/opcodes/attributes.rs`
        - [ ] Implement `op_get_attr()` with helper functions:
            - [ ] `get_instance_attr()`
            - [ ] `get_class_attr()`
            - [ ] `get_super_proxy_attr()`
            - [ ] `get_function_attr()`
            - [ ] `get_function_prototype_attr()`
        - [ ] Add to `opcodes/mod.rs`
        - [ ] Update dispatcher to call new function
        - [ ] Run `cargo test`
    - [ ] 2d: Extract OpSetAttr (~16 lines)
        - [ ] Add `op_set_attr()` to `opcodes/attributes.rs`
        - [ ] Update dispatcher to call new function
        - [ ] Run `cargo test`
    - [ ] 2e: Extract OpInherit (~26 lines)
        - [ ] Add `op_inherit()` to `opcodes/classes.rs`
        - [ ] Update dispatcher to call new function
        - [ ] Run `cargo test`
    - [ ] Run `cargo fmt && cargo clippy`

- [ ] Phase 3: Extract Function Call Logic (~175 lines)
    - [ ] Create `src/vm/function_calls.rs`
    - [ ] Move `call_value()` method
    - [ ] Move `call_function()` method
    - [ ] Add helper methods for call types
    - [ ] Add module declaration to `mod.rs`
    - [ ] Run `cargo fmt && cargo clippy`
    - [ ] Run `cargo test`

- [ ] Phase 4: Extract Return Handling (~43 lines)
    - [ ] Create `src/vm/return_handler.rs`
    - [ ] Move `handle_return()` method
    - [ ] Add module declaration to `mod.rs`
    - [ ] Run `cargo fmt && cargo clippy`
    - [ ] Run `cargo test`

- [ ] Phase 5: Simplify Main run() Loop
    - [ ] Refactor `run()` to simplified version
    - [ ] Adjust dispatcher control flow if needed
    - [ ] Run `cargo fmt && cargo clippy`
    - [ ] Run `cargo test`

- [ ] Phase 6: Final Verification
    - [ ] Verify `mod.rs` is ~100-150 lines
    - [ ] Verify all tests pass: `cargo test`
    - [ ] Verify no clippy warnings: `cargo clippy`
    - [ ] Verify 100% test coverage: `cargo tarpaulin`
    - [ ] Verify code formatting: `cargo fmt --check`
    - [ ] Update README.md to mark refactoring as complete
