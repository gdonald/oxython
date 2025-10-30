// Function-related opcodes
//
// OpMakeFunction - handled inline in VM (needs frame and upvalue access)
// OpCall - handled inline in VM (calls VM::call_value method)
// OpReturn - handled inline in VM (calls VM::handle_return method)
//
// Note: Function operations are deeply integrated with the VM's call frame
// management and remain in the main VM run() loop.
