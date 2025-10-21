use crate::object::Object;

/// Represents the instructions that our Virtual Machine will execute.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    /// Pushes a constant from the chunk's constant pool onto the stack.
    OpConstant,
    /// Pops two values from the stack, adds them, and pushes the result.
    OpAdd,
    /// Pops two values and divides the first by the second.
    OpDivide,
    /// Pops two values, subtracts the second from the first, and pushes the result.
    OpSubtract,
    /// Defines a new global variable.
    OpDefineGlobal,
    /// Pushes the value of a global variable onto the stack.
    OpGetGlobal,
    /// Sets the value of an existing global variable.
    OpSetGlobal,
    /// Pops a value, prints it with a trailing space.
    OpPrintSpaced,
    /// Pops a value and prints it without a trailing space.
    OpPrint,
    /// Signals the end of execution.
    OpReturn,
    /// Pops the top value from the stack.
    OpPop,
    /// Prints a newline character.
    OpPrintln,
    /// Indexes into a list.
    OpIndex,
    /// Computes the length of a list or string.
    OpLen,
    /// Appends a value to a list.
    OpAppend,
    /// Rounds a floating point number to a given precision.
    OpRound,
    /// Advances an iterator over a collection.
    OpIterNext,
    /// Jumps backwards by a given offset.
    OpLoop,
    /// Jumps forward if the top of the stack is falsy.
    OpJumpIfFalse,
    /// Unconditionally jumps forward by an offset.
    OpJump,
    /// Sets a value at an index/key in a collection.
    OpSetIndex,
    /// Duplicates the top stack value.
    OpDup,
    /// Tests membership of a value in a collection.
    OpContains,
    /// Swaps the top two stack values.
    OpSwap,
    /// Pops two values and multiplies them.
    OpMultiply,
    /// Builds a list representing a numeric range.
    OpRange,
    /// Compares two numeric values and pushes a boolean indicating if left < right.
    OpLess,
    /// Extracts a slice from a list or string.
    OpSlice,
    /// Calculates the modulo of two integers.
    OpModulo,
    /// Compares two values for equality.
    OpEqual,
    /// Converts a value into a list.
    OpToList,
    /// Zips multiple iterables together.
    OpZip,
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        match byte {
            0 => OpCode::OpConstant,
            1 => OpCode::OpAdd,
            2 => OpCode::OpDivide,
            3 => OpCode::OpSubtract,
            4 => OpCode::OpDefineGlobal,
            5 => OpCode::OpGetGlobal,
            6 => OpCode::OpSetGlobal,
            7 => OpCode::OpPrintSpaced,
            8 => OpCode::OpPrint,
            9 => OpCode::OpReturn,
            10 => OpCode::OpPop,
            11 => OpCode::OpPrintln,
            12 => OpCode::OpIndex,
            13 => OpCode::OpLen,
            14 => OpCode::OpAppend,
            15 => OpCode::OpRound,
            16 => OpCode::OpIterNext,
            17 => OpCode::OpLoop,
            18 => OpCode::OpJumpIfFalse,
            19 => OpCode::OpJump,
            20 => OpCode::OpSetIndex,
            21 => OpCode::OpDup,
            22 => OpCode::OpContains,
            23 => OpCode::OpSwap,
            24 => OpCode::OpMultiply,
            25 => OpCode::OpRange,
            26 => OpCode::OpLess,
            27 => OpCode::OpSlice,
            28 => OpCode::OpModulo,
            29 => OpCode::OpEqual,
            30 => OpCode::OpToList,
            31 => OpCode::OpZip,
            _ => panic!("Invalid opcode: {}", byte),
        }
    }
}
/// A chunk of bytecode representing a compiled script or function.
#[derive(Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Object>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
