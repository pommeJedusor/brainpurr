use std::{fs, path::PathBuf};

use crate::parser::Instruction;

const INSTRUCTIONS: [char; 8] = ['>', '<', '+', '-', '.', ',', '[', ']'];


pub fn parse_file(file_path: &PathBuf) -> Vec<Instruction> {
    let content = fs::read_to_string(file_path)
        .expect("failed to read file");

    let instructions_words = content.chars().filter(|x| INSTRUCTIONS.contains(x));

    let mut open_brackets = vec![];
    let mut close_brackets = vec![];
    let mut instructions = instructions_words.enumerate().map(|(index, instruction_word)| match instruction_word {
        '>' => Instruction::PointerIncrement,
        '<' => Instruction::PointerDecrement,
        '+' => Instruction::ByteIncrement,
        '-' => Instruction::ByteDecrement,
        '.' => Instruction::ByteOutput,
        ',' => Instruction::ByteInput,
        '[' => {
            open_brackets.push(index);
            Instruction::OpenLoop(0)
        },
        ']' => {
            let open_bracket_index = open_brackets.pop().expect("found a ] without its required [");
            close_brackets.push((open_bracket_index, index));
            Instruction::CloseLoop(open_bracket_index)
        },
        _ => unreachable!(),
    }).collect::<Vec<Instruction>>();

    assert!(open_brackets.len() == 0, "found a [ without its required ]");

    for (open_bracket_index, close_bracket_index) in close_brackets {
        instructions[open_bracket_index] = Instruction::OpenLoop(close_bracket_index);
    }

    instructions
}

pub fn unparse(instructions: Vec<Instruction>) -> String{
    instructions.iter().map(|instruction| match instruction{
        Instruction::PointerIncrement => '>',
        Instruction::PointerDecrement => '<',
        Instruction::ByteIncrement => '+',
        Instruction::ByteDecrement => '-',
        Instruction::ByteOutput => '.',
        Instruction::ByteInput => ',',
        Instruction::OpenLoop(_) => '[',
        Instruction::CloseLoop(_) => ']',
    }).collect::<String>()
}
