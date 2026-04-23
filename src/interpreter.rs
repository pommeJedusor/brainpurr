use std::io;

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

pub fn interpreter(instructions: Vec<Instruction>) {
    let mut array = [0u8; 67_000];
    let mut array_pointer = 0;
    let mut instruction_pointer = 0;

    while instruction_pointer < instructions.len(){
        match instructions[instruction_pointer] {
            Instruction::PointerIncrement => array_pointer += 1,
            Instruction::PointerDecrement => array_pointer -= 1,
            Instruction::ByteIncrement => array[array_pointer] = array[array_pointer].wrapping_add(1),
            Instruction::ByteDecrement => array[array_pointer] = array[array_pointer].wrapping_sub(1),
            Instruction::ByteInput => {
              let mut input = String::new();
              io::stdin().read_line(&mut input).expect("error: unable to read user input");
              let input = input.chars().next().unwrap() as u8;
              array[array_pointer] = input;
            },
            Instruction::ByteOutput => println!("{}", array[array_pointer] as char),
            Instruction::OpenLoop(close_loop_index) => if array[array_pointer] == 0{instruction_pointer = close_loop_index},
            Instruction::CloseLoop(open_loop_index) => if array[array_pointer] != 0{instruction_pointer = open_loop_index},
        }

        instruction_pointer += 1;
    }
}
