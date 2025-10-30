// Variable access opcodes
//
// OpGetLocal, OpSetLocal - handled inline in VM (needs frame and stack access)
// OpGetGlobal, OpSetGlobal, OpDefineGlobal - handled inline in VM (needs globals HashMap)
// OpGetUpvalue, OpSetUpvalue - handled inline in VM (needs upvalue management)
//
// Note: Variable operations are tightly coupled with VM state (frames, globals, upvalues)
// and remain in the main VM run() loop for efficiency and simplicity.
