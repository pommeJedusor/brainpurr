use std::fs;

const INSTRUCTIONS: [&str; 8] = ["meow", "mrow", "mrp", "purr", ":3c", ">:3", "nya", ":3"];

#[derive(Debug)]
pub enum Instruction {
    PointerIncrement,
    PointerDecrement,
    ByteIncrement,
    ByteDecrement,
    ByteOutput,
    ByteInput,
    OpenLoop(usize),
    CloseLoop(usize),
}


pub fn parse_file(file_path: &str) -> Vec<Instruction> {
    let content = fs::read_to_string(file_path)
        .expect("failed to read file");

    let instructions_words = content.split_whitespace().filter(|x| INSTRUCTIONS.contains(x));

    let mut open_brackets = vec![];
    let mut close_brackets = vec![];
    let mut instructions = instructions_words.enumerate().map(|(index, instruction_word)| match instruction_word {
        "meow" => Instruction::PointerIncrement,
        "mrow" => Instruction::PointerDecrement,
        "mrp" => Instruction::ByteIncrement,
        "purr" => Instruction::ByteDecrement,
        ":3c" => Instruction::ByteOutput,
        ">:3" => Instruction::ByteInput,
        "nya" => {
            open_brackets.push(index);
            Instruction::OpenLoop(0)
        },
        ":3" => {
            let open_bracket_index = open_brackets.pop().expect("found a :3 without its required nya");
            close_brackets.push((open_bracket_index, index));
            Instruction::CloseLoop(open_bracket_index)
        },
        _ => unreachable!(),
    }).collect::<Vec<Instruction>>();

    assert!(open_brackets.len() == 0, "found a nya without its required :3");

    for (open_bracket_index, close_bracket_index) in close_brackets {
        instructions[open_bracket_index] = Instruction::OpenLoop(close_bracket_index);
    }

    instructions
}
