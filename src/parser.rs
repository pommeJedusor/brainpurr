use std::{fs, path::PathBuf};

use crate::Error;

const INSTRUCTIONS: [&str; 8] = ["meow", "mrow", "mrp", "purr", ":3c", ">:3", "nya", ":3"];

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Instruction {
    PointerIncrement(usize),
    PointerDecrement(usize),
    ByteIncrement(usize),
    ByteDecrement(usize),
    ByteOutput,
    ByteInput,
    OpenLoop(usize),
    CloseLoop(usize),
}

fn optimize_instructions_pointer(mut instructions: Vec<Instruction>) -> Vec<Instruction> {
    let mut pointer_crement = instructions
        .iter()
        .enumerate()
        .filter(|(_, x)| {
            matches!(
                x,
                Instruction::PointerIncrement(_) | Instruction::PointerDecrement(_)
            )
        })
        .map(|(i, x)| {
            (
                i,
                match x {
                    Instruction::PointerIncrement(x) => *x as i32,
                    Instruction::PointerDecrement(x) => -(*x as i32),
                    _ => unreachable!(),
                },
            )
        })
        .collect::<Vec<(usize, i32)>>();

    for i in 0..pointer_crement.len() {
        if i != 0 && pointer_crement[i - 1].0 + 1 == pointer_crement[i].0 {
            pointer_crement[i].1 += pointer_crement[i - 1].1;
            pointer_crement[i - 1].1 = 0;
        }
    }

    pointer_crement.iter().for_each(|(i, x)| {
        instructions[*i] = match x {
            0.. => Instruction::PointerIncrement(*x as usize),
            ..0 => Instruction::PointerDecrement(-*x as usize),
        }
    });

    instructions
}

fn optimize_instructions_byte(mut instructions: Vec<Instruction>) -> Vec<Instruction> {
    let mut byte_crement = instructions
        .iter()
        .enumerate()
        .filter(|(_, x)| {
            matches!(
                x,
                Instruction::ByteIncrement(_) | Instruction::ByteDecrement(_)
            )
        })
        .map(|(i, x)| {
            (
                i,
                match x {
                    Instruction::ByteIncrement(x) => *x as i32,
                    Instruction::ByteDecrement(x) => -(*x as i32),
                    _ => unreachable!(),
                },
            )
        })
        .collect::<Vec<(usize, i32)>>();

    for i in 0..byte_crement.len() {
        if i != 0 && byte_crement[i - 1].0 + 1 == byte_crement[i].0 {
            byte_crement[i].1 += byte_crement[i - 1].1;
            byte_crement[i - 1].1 = 0;
        }
    }

    byte_crement.iter().for_each(|(i, x)| {
        instructions[*i] = match x {
            0.. => Instruction::ByteIncrement(*x as usize),
            ..0 => Instruction::ByteDecrement(-*x as usize),
        }
    });

    instructions
}

pub fn optimize_instructions(mut instructions: Vec<Instruction>) -> Vec<Instruction> {
    let mut previous_instructions_length = None;
    while previous_instructions_length.is_none_or(|x| x != instructions.len()) {
        previous_instructions_length = Some(instructions.len());
        instructions = optimize_instructions_byte(optimize_instructions_pointer(instructions))
            .iter()
            .filter(|x| {
                !matches!(
                    x,
                    Instruction::ByteIncrement(0)
                        | Instruction::ByteDecrement(0)
                        | Instruction::PointerIncrement(0)
                        | Instruction::PointerDecrement(0)
                )
            })
            .map(|x| x.to_owned())
            .collect();
    }

    instructions
}

pub fn parse_file(file_path: &PathBuf) -> Result<Vec<Instruction>, Error> {
    parse(&fs::read_to_string(file_path)?)
}

fn get_instructions(program: &str) -> Vec<Instruction> {
    let instructions_words = program
        .split_whitespace()
        .filter(|x| INSTRUCTIONS.contains(x));

    instructions_words
        .map(|instruction_word| match instruction_word {
            "meow" => Instruction::PointerIncrement(1),
            "mrow" => Instruction::PointerDecrement(1),
            "mrp" => Instruction::ByteIncrement(1),
            "purr" => Instruction::ByteDecrement(1),
            ":3c" => Instruction::ByteOutput,
            ">:3" => Instruction::ByteInput,
            "nya" => Instruction::OpenLoop(0),
            ":3" => Instruction::CloseLoop(0),
            _ => unreachable!(),
        })
        .collect::<Vec<Instruction>>()
}

pub fn fix_instructions(instructions: Vec<Instruction>) -> Result<Vec<Instruction>, Error> {
    let mut instructions = optimize_instructions(instructions);

    let mut open_brackets = vec![];
    let mut close_brackets = vec![];
    for (i, instruction) in instructions.iter().enumerate() {
        match instruction {
            Instruction::OpenLoop(_) => {
                open_brackets.push(i);
            }
            Instruction::CloseLoop(_) => {
                let open_bracket_index = open_brackets.pop();
                match open_bracket_index {
                    Some(open_bracket_index) => close_brackets.push((open_bracket_index, i)),
                    None => {
                        return Err(Error::TooManyCloseLoop);
                    }
                };
            }
            _ => {}
        };
    }

    if !open_brackets.is_empty() {
        return Err(Error::TooManyOpenLoop);
    }

    for (open_bracket_index, close_bracket_index) in close_brackets {
        instructions[open_bracket_index] = Instruction::OpenLoop(close_bracket_index);
        instructions[close_bracket_index] = Instruction::CloseLoop(open_bracket_index);
    }

    Ok(instructions)
}

pub fn parse(program: &str) -> Result<Vec<Instruction>, Error> {
    fix_instructions(get_instructions(program))
}

pub fn unparse(instructions: Vec<Instruction>) -> String {
    instructions
        .iter()
        .map(|instruction| match instruction {
            Instruction::PointerIncrement(x) => vec!["meow"; *x].join(" "),
            Instruction::PointerDecrement(x) => vec!["mrow"; *x].join(" "),
            Instruction::ByteIncrement(x) => vec!["mrp"; *x].join(" "),
            Instruction::ByteDecrement(x) => vec!["purr"; *x].join(" "),
            Instruction::ByteOutput => ":3c".to_string(),
            Instruction::ByteInput => ">:3".to_string(),
            Instruction::OpenLoop(_) => "nya".to_string(),
            Instruction::CloseLoop(_) => ":3".to_string(),
        })
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        assert_eq!(
            parse("meow mrp mrow purr :3c >:3 nya :3").unwrap(),
            vec![
                Instruction::PointerIncrement(1),
                Instruction::ByteIncrement(1),
                Instruction::PointerDecrement(1),
                Instruction::ByteDecrement(1),
                Instruction::ByteOutput,
                Instruction::ByteInput,
                Instruction::OpenLoop(7),
                Instruction::CloseLoop(6)
            ]
        );
    }

    #[test]
    fn unparsing() {
        assert_eq!(
            unparse(vec![
                Instruction::PointerIncrement(1),
                Instruction::PointerDecrement(1),
                Instruction::ByteIncrement(1),
                Instruction::ByteDecrement(1),
                Instruction::ByteOutput,
                Instruction::ByteInput,
                Instruction::OpenLoop(7),
                Instruction::CloseLoop(6)
            ]),
            "meow mrow mrp purr :3c >:3 nya :3"
        );
    }

    #[test]
    #[should_panic]
    fn too_many_open_loop() {
        parse("nya nya :3").unwrap();
    }

    #[test]
    #[should_panic]
    fn too_many_close_loop() {
        parse("nya :3 :3").unwrap();
    }

    #[test]
    fn optimizations() {
        assert_eq!(
            parse("meow mrp meow mrp meow mrp purr mrow purr mrow purr mrow").unwrap(),
            vec![]
        );
    }
}
