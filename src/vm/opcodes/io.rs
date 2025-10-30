// I/O opcodes
//
// OpPrint, OpPrintln, OpPrintSpaced - handled inline in VM
// These opcodes are tightly coupled with VM state (get_string_representation method)
// and remain in the main VM run() loop.
//
// Note: Print operations require access to VM's get_string_representation method
// which itself needs VM state, so these remain inline.
