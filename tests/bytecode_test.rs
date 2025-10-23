use oxython::bytecode::{Chunk, OpCode};

#[test]
fn test_opcode_from_u8() {
    assert_eq!(OpCode::from(0), OpCode::OpConstant);
    assert_eq!(OpCode::from(1), OpCode::OpAdd);
    assert_eq!(OpCode::from(2), OpCode::OpDivide);
    assert_eq!(OpCode::from(3), OpCode::OpSubtract);
    assert_eq!(OpCode::from(4), OpCode::OpDefineGlobal);
    assert_eq!(OpCode::from(5), OpCode::OpGetGlobal);
    assert_eq!(OpCode::from(6), OpCode::OpSetGlobal);
    assert_eq!(OpCode::from(7), OpCode::OpPrintSpaced);
    assert_eq!(OpCode::from(8), OpCode::OpPrint);
    assert_eq!(OpCode::from(9), OpCode::OpReturn);
    assert_eq!(OpCode::from(10), OpCode::OpPop);
    assert_eq!(OpCode::from(11), OpCode::OpPrintln);
    assert_eq!(OpCode::from(12), OpCode::OpIndex);
    assert_eq!(OpCode::from(13), OpCode::OpLen);
    assert_eq!(OpCode::from(14), OpCode::OpAppend);
    assert_eq!(OpCode::from(15), OpCode::OpRound);
    assert_eq!(OpCode::from(16), OpCode::OpIterNext);
    assert_eq!(OpCode::from(17), OpCode::OpLoop);
    assert_eq!(OpCode::from(18), OpCode::OpJumpIfFalse);
    assert_eq!(OpCode::from(19), OpCode::OpJump);
    assert_eq!(OpCode::from(20), OpCode::OpSetIndex);
    assert_eq!(OpCode::from(21), OpCode::OpDup);
    assert_eq!(OpCode::from(22), OpCode::OpContains);
    assert_eq!(OpCode::from(23), OpCode::OpSwap);
    assert_eq!(OpCode::from(24), OpCode::OpMultiply);
    assert_eq!(OpCode::from(25), OpCode::OpRange);
    assert_eq!(OpCode::from(26), OpCode::OpLess);
    assert_eq!(OpCode::from(27), OpCode::OpSlice);
    assert_eq!(OpCode::from(35), OpCode::OpCall);
    assert_eq!(OpCode::from(36), OpCode::OpGetLocal);
    assert_eq!(OpCode::from(37), OpCode::OpSetLocal);
    assert_eq!(OpCode::from(38), OpCode::OpMakeFunction);
}

#[test]
#[should_panic(expected = "Invalid opcode")]
fn test_opcode_from_u8_invalid_panics() {
    let _ = OpCode::from(255);
}

#[test]
fn test_chunk_default_is_empty() {
    let chunk = Chunk::default();
    assert!(chunk.code.is_empty());
    assert!(chunk.constants.is_empty());
}
