use std::{fs, path::PathBuf};

use crate::parser::{Instruction, fix_instructions};

const INSTRUCTIONS: [char; 8] = ['>', '<', '+', '-', '.', ',', '[', ']'];


pub fn parse_file(file_path: &PathBuf) -> Vec<Instruction> {
    let content = fs::read_to_string(file_path)
        .expect("failed to read file");
    parse(&content)
}

fn get_instructions(program: &str) -> Vec<Instruction> {
    let instructions_words = program.chars().filter(|x| INSTRUCTIONS.contains(x));

    instructions_words.map(|instruction_word| match instruction_word {
        '>' => Instruction::PointerIncrement(1),
        '<' => Instruction::PointerDecrement(1),
        '+' => Instruction::ByteIncrement(1),
        '-' => Instruction::ByteDecrement(1),
        '.' => Instruction::ByteOutput,
        ',' => Instruction::ByteInput,
        '[' => Instruction::OpenLoop(0),
        ']' => Instruction::CloseLoop(0),
        _ => unreachable!(),
    }).collect::<Vec<Instruction>>()
}

pub fn parse(program: &str) -> Vec<Instruction> {
    fix_instructions(get_instructions(program))
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
        assert_eq!(parse(">+<-.,[]"), vec![Instruction::PointerIncrement(1), Instruction::ByteIncrement(1), Instruction::PointerDecrement(1), Instruction::ByteDecrement(1), Instruction::ByteOutput, Instruction::ByteInput, Instruction::OpenLoop(7), Instruction::CloseLoop(6)]);
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

    #[test]
    fn optimizations(){
        assert_eq!(parse(">+>+>+-<-<-<"),vec![]);
    }
}
