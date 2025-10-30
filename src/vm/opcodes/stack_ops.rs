// Stack manipulation opcodes
//
// OpConstant - handled inline in VM (needs access to chunk constants)
// OpPop - handled inline in VM (simple pop operation)
// OpDup - handled inline in VM (simple peek and push)
// OpSwap - handled inline in VM (needs direct stack access)
//
// Note: These opcodes are simple enough that they remain in the main VM run() loop
// rather than being extracted to separate handlers.
