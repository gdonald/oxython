// Class and OOP-related opcodes
//
// OpMakeClass - handled inline in VM (complex class construction with methods)
// OpGetAttr, OpSetAttr - handled inline in VM (needs instance access and method binding)
// OpInherit - handled inline in VM (class hierarchy manipulation)
//
// Note: OOP operations are tightly coupled with object model and VM state,
// and remain in the main VM run() loop.
