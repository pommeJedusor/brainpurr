use std::io::{self, Write};

use crate::{InputMethod, OutputMethod, parser::Instruction};

pub fn interpreter(instructions: Vec<Instruction>, input_method: &InputMethod, output_method: &OutputMethod, mut writer: impl std::io::Write) -> Vec<u8> {
    let mut array: Vec<u8> = vec![0];
    let mut array_pointer = 0;
    let mut instruction_pointer = 0;
    let mut input_queue = vec![];

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
                match input_method {
                    InputMethod::Normal => {
                        while input_queue.len() == 0 {
                            let mut input = String::new();
                            io::stdin().read_line(&mut input).expect("error: unable to read user input");
                            input_queue = input.into_bytes();
                            input_queue.reverse();
                        }
                        array[array_pointer] = input_queue.pop().unwrap();
                    },
                    InputMethod::FirstCharOnly => {
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).expect("error: unable to read user input");
                        let input = input.chars().next().unwrap() as u8;
                        array[array_pointer] = input;
                    },
                    InputMethod::ByteAsNumber => {
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).expect("error: unable to read user input");
                        let input = input[0..input.len() - 1].parse().expect("error: unable to parse user input into a single byte");
                        array[array_pointer] = input;
                    },
                };
            },
            Instruction::ByteOutput => match output_method{
                OutputMethod::Normal => {
                    let _ = write!(writer, "{}", array[array_pointer] as char);
                    io::stdout().flush().unwrap();
                },
                OutputMethod::ByteAsNumber => {
                    let _ = writeln!(writer, "{}", array[array_pointer]);
                },
            },
            Instruction::OpenLoop(close_loop_index) => if array[array_pointer] == 0{instruction_pointer = close_loop_index},
            Instruction::CloseLoop(open_loop_index) => if array[array_pointer] != 0{instruction_pointer = open_loop_index},
        }

        instruction_pointer += 1;
    }

    array
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment(){
        assert_eq!(interpreter(vec![Instruction::ByteIncrement], &InputMethod::Normal, &OutputMethod::Normal, &mut vec![]), vec![1]);
    }

    #[test]
    fn increment_overflow(){
        assert_eq!(interpreter(vec![Instruction::ByteIncrement; 256], &InputMethod::Normal, &OutputMethod::Normal, &mut vec![]), vec![0]);
    }

    #[test]
    fn decrement(){
        assert_eq!(interpreter(vec![Instruction::ByteIncrement, Instruction::ByteDecrement], &InputMethod::Normal, &OutputMethod::Normal, &mut vec![]), vec![0]);
    }

    #[test]
    fn decrement_overflow(){
        assert_eq!(interpreter(vec![Instruction::ByteDecrement; 256], &InputMethod::Normal, &OutputMethod::Normal, &mut vec![]), vec![0]);
    }

    #[test]
    fn pointer_increment(){
        assert_eq!(interpreter(vec![Instruction::PointerIncrement, Instruction::ByteIncrement], &InputMethod::Normal, &OutputMethod::Normal, &mut vec![]), vec![0, 1]);
    }

    #[test]
    fn pointer_decrement(){
        assert_eq!(interpreter(vec![Instruction::PointerIncrement, Instruction::PointerDecrement, Instruction::ByteIncrement], &InputMethod::Normal, &OutputMethod::Normal, &mut vec![]), vec![1, 0]);
    }

    #[test]
    #[should_panic]
    fn pointer_decrement_overflow(){
        interpreter(vec![Instruction::PointerDecrement], &InputMethod::Normal, &OutputMethod::Normal, &mut vec![]);
    }

    #[test]
    fn output_normal_mode(){
        let mut instructions = vec![Instruction::ByteIncrement; 67];
        instructions.push(Instruction::ByteOutput);
        let mut result = vec![];
        interpreter(instructions, &InputMethod::Normal, &OutputMethod::Normal, &mut result);
        assert_eq!(result, vec![67])
    }

    #[test]
    fn output_byte_as_number(){
        let mut instructions = vec![Instruction::ByteIncrement; 67];
        instructions.push(Instruction::ByteOutput);
        let mut result = vec![];
        interpreter(instructions, &InputMethod::Normal, &OutputMethod::ByteAsNumber, &mut result);
        assert_eq!(result, vec!['6' as u8, '7' as u8, '\n' as u8])
    }
}
