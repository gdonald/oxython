// Control flow opcodes
//
// OpJump, OpJumpIfFalse, OpLoop - handled inline in VM (needs frame IP manipulation)
// OpIterNext - handled inline in VM (complex logic with stack and frame access)
//
// Note: Control flow operations manipulate the instruction pointer directly
// and remain in the main VM run() loop for performance and direct frame access.
