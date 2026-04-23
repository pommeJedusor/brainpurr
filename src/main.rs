mod interpreter;
use crate::interpreter::*;

fn main() {
    let instructions = vec![
        Instruction::ByteInput,
        Instruction::PointerIncrement,
        Instruction::ByteInput,
        Instruction::PointerDecrement,

        // [->+<]
        Instruction::OpenLoop(8),
        Instruction::ByteDecrement,
        Instruction::PointerIncrement,
        Instruction::ByteIncrement,
        Instruction::PointerDecrement,
        Instruction::CloseLoop(3),

        Instruction::ByteOutput,
        Instruction::PointerIncrement,
        Instruction::ByteOutput,
    ];
    interpreter(instructions);
}
