use std::io;

use crate::parser::Instruction;

pub fn interpreter(instructions: Vec<Instruction>) -> Vec<u8> {
    let mut array: Vec<u8> = vec![0];
    let mut array_pointer = 0;
    let mut instruction_pointer = 0;

    while instruction_pointer < instructions.len(){
        match instructions[instruction_pointer] {
            Instruction::PointerIncrement => {
                if array.len() == array_pointer + 1 {
                    array.push(0);
                }
                array_pointer += 1
            },
            Instruction::PointerDecrement => array_pointer -= 1,
            Instruction::ByteIncrement => array[array_pointer] = array[array_pointer].wrapping_add(1),
            Instruction::ByteDecrement => array[array_pointer] = array[array_pointer].wrapping_sub(1),
            Instruction::ByteInput => {
              let mut input = String::new();
              io::stdin().read_line(&mut input).expect("error: unable to read user input");
              let input = input.chars().next().unwrap() as u8;
              array[array_pointer] = input;
            },
            Instruction::ByteOutput => print!("{}", array[array_pointer] as char),
            Instruction::OpenLoop(close_loop_index) => if array[array_pointer] == 0{instruction_pointer = close_loop_index},
            Instruction::CloseLoop(open_loop_index) => if array[array_pointer] != 0{instruction_pointer = open_loop_index},
        }

        instruction_pointer += 1;
    }

    array
}
