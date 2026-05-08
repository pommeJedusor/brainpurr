use std::{fs, path::PathBuf};

use crate::parser::Instruction;

const INSTRUCTIONS: [char; 8] = ['>', '<', '+', '-', '.', ',', '[', ']'];


pub fn parse_file(file_path: &PathBuf) -> Vec<Instruction> {
    let content = fs::read_to_string(file_path)
        .expect("failed to read file");
    parse(&content)
}

// TODO add optimization to bf_parser
pub fn parse(program: &str) -> Vec<Instruction> {
    let instructions_words = program.chars().filter(|x| INSTRUCTIONS.contains(x));

    let mut open_brackets = vec![];
    let mut close_brackets = vec![];
    let mut instructions = instructions_words.enumerate().map(|(index, instruction_word)| match instruction_word {
        '>' => Instruction::PointerIncrement(1),
        '<' => Instruction::PointerDecrement(1),
        '+' => Instruction::ByteIncrement(1),
        '-' => Instruction::ByteDecrement(1),
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
        Instruction::PointerIncrement(x) => '>'.to_string().repeat(*x),
        Instruction::PointerDecrement(x) => '<'.to_string().repeat(*x),
        Instruction::ByteIncrement(x) => '+'.to_string().repeat(*x),
        Instruction::ByteDecrement(x) => '-'.to_string().repeat(*x),
        Instruction::ByteOutput => '.'.to_string(),
        Instruction::ByteInput => ','.to_string(),
        Instruction::OpenLoop(_) => '['.to_string(),
        Instruction::CloseLoop(_) => ']'.to_string(),
    }).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing(){
        assert_eq!(parse("><+-.,[]"), vec![Instruction::PointerIncrement(1), Instruction::PointerDecrement(1), Instruction::ByteIncrement(1), Instruction::ByteDecrement(1), Instruction::ByteOutput, Instruction::ByteInput, Instruction::OpenLoop(7), Instruction::CloseLoop(6)]);
    }

    #[test]
    fn unparsing(){
        assert_eq!(unparse(vec![Instruction::PointerIncrement(1), Instruction::PointerDecrement(1), Instruction::ByteIncrement(1), Instruction::ByteDecrement(1), Instruction::ByteOutput, Instruction::ByteInput, Instruction::OpenLoop(7), Instruction::CloseLoop(6)]), "><+-.,[]");
    }

    #[test]
    #[should_panic]
    fn too_many_open_loop(){
        parse("[[]");
    }

    #[test]
    #[should_panic]
    fn too_many_close_loop(){
        parse("[]]");
    }
}
