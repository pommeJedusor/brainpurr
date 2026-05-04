use std::{io::{self, Write}, process};

use crate::{InputMethod, OutputMethod, args::InterpreterArgs, parser::Instruction};

fn newline_zero_map(char: u8) -> u8{
    match char{
        0 => 10,
        10 => 0,
        _ => char,
    }
}
fn useless_map(char: u8) -> u8{
    char
}

pub fn interpreter<T: InterpreterArgs>(instructions: Vec<Instruction>, args: &T, mut writer: impl std::io::Write) -> Vec<u8> {
    let input_method = args.get_input_method();
    let output_method = args.get_output_method();
    let newline_zero = args.get_newline_zero();
    let max_array_size = args.get_max_array_size();
    
    let newline_zero_func = if newline_zero {newline_zero_map} else {useless_map};

    let mut array: Vec<u8> = vec![0];
    if let Some(max_array_size) = max_array_size {
        array = vec![0; max_array_size as usize];
    }
    let mut array_pointer = 0;
    let mut instruction_pointer = 0;
    let mut input_queue = vec![];

    while instruction_pointer < instructions.len(){
        match instructions[instruction_pointer] {
            Instruction::PointerIncrement(x) => {
                if array.len() == array_pointer + x && max_array_size.is_none() {
                    array.push(0);
                }else if array.len() == array_pointer + x {
                    eprintln!("pointer overflow");
                    process::exit(1);
                }
                array_pointer += x
            },
            Instruction::PointerDecrement(x) => array_pointer -= x,
            Instruction::ByteIncrement(x) => array[array_pointer] = array[array_pointer].wrapping_add((x % 256) as u8),
            Instruction::ByteDecrement(x) => array[array_pointer] = array[array_pointer].wrapping_sub((x % 256) as u8),
            Instruction::ByteInput => {
                match input_method {
                    InputMethod::Normal => {
                        while input_queue.len() == 0 {
                            let mut input = String::new();
                            io::stdin().read_line(&mut input).expect("error: unable to read user input");
                            input_queue = input.into_bytes().iter().rev().map(|x| newline_zero_func(*x)).collect();
                        }
                        array[array_pointer] = input_queue.pop().unwrap();
                    },
                    InputMethod::FirstCharOnly => {
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).expect("error: unable to read user input");
                        let input = input.chars().map(|x| newline_zero_func(x as u8)).next().unwrap() as u8;
                        array[array_pointer] = input;
                    },
                    InputMethod::ByteAsNumber => {
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).expect("error: unable to read user input");
                        let input = input[0..input.len() - 1].parse().expect("error: unable to parse user input into a single byte");
                        let input = newline_zero_func(input);
                        array[array_pointer] = input;
                    },
                };
            },
            Instruction::ByteOutput => match output_method{
                OutputMethod::Normal => {
                    let _ = write!(writer, "{}", newline_zero_func(array[array_pointer]) as char);
                    io::stdout().flush().unwrap();
                },
                OutputMethod::ByteAsNumber => {
                    let _ = writeln!(writer, "{}", newline_zero_func(array[array_pointer]));
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
    use assert_cmd::Command;

    struct Args{
        input: InputMethod,
        output: OutputMethod,
        newline_zero: bool,
        max_array_size: Option<u32>,
    }

    impl InterpreterArgs for Args {
        fn get_input_method(&self) -> &InputMethod { &self.input }
        fn get_output_method(&self) -> &OutputMethod { &self.output }
        fn get_newline_zero(&self) -> bool { self.newline_zero }
        fn get_max_array_size(&self) -> Option<u32> { self.max_array_size }
    }

    impl Args {
        fn new(input: InputMethod, output: OutputMethod, newline_zero: bool, max_array_size: Option<u32>) -> Self{
            Self { input, output, newline_zero, max_array_size }
        }
    }

    #[test]
    fn increment(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false, None);
        assert_eq!(interpreter(vec![Instruction::ByteIncrement(1)], &args, &mut vec![]), vec![1]);
    }

    #[test]
    fn byte_overflow(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false, None);
        assert_eq!(interpreter(vec![Instruction::ByteIncrement(1); 256], &args, &mut vec![]), vec![0]);
    }
    #[test]
    fn increment_overflow(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false, None);
        assert_eq!(interpreter(vec![Instruction::ByteIncrement(256)], &args, &mut vec![]), vec![0]);
    }

    #[test]
    fn decrement(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false, None);
        assert_eq!(interpreter(vec![Instruction::ByteIncrement(1), Instruction::ByteDecrement(1)], &args, &mut vec![]), vec![0]);
    }

    #[test]
    fn byte_underflow(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false, None);
        assert_eq!(interpreter(vec![Instruction::ByteDecrement(1); 256], &args, &mut vec![]), vec![0]);
    }
    #[test]
    fn decrement_overflow(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false, None);
        assert_eq!(interpreter(vec![Instruction::ByteDecrement(256)], &args, &mut vec![]), vec![0]);
    }

    #[test]
    fn pointer_increment(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false, None);
        assert_eq!(interpreter(vec![Instruction::PointerIncrement(1), Instruction::ByteIncrement(1)], &args, &mut vec![]), vec![0, 1]);
    }

    #[test]
    fn pointer_decrement(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false, None);
        assert_eq!(interpreter(vec![Instruction::PointerIncrement(1), Instruction::PointerDecrement(1), Instruction::ByteIncrement(1)], &args, &mut vec![]), vec![1, 0]);
    }

    #[test]
    #[should_panic]
    fn pointer_underflow(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false, None);
        interpreter(vec![Instruction::PointerDecrement(1)], &args, &mut vec![]);
    }

    #[test]
    fn input_normal_mode(){
        let mut cmd = Command::cargo_bin("brainpurr").unwrap();
        let _ = cmd
            .args(&["./examples/echo.bp"])
            .write_stdin("pomme is cute\n")
            .assert()
            .code(0)
            .stdout("pomme is cute\n");
    }

    #[test]
    fn input_first_char_only_mode(){
        let mut cmd = Command::cargo_bin("brainpurr").unwrap();
        let _ = cmd
            .args(&["./examples/echo.bp", "--input", "first-char-only"])
            .write_stdin("pomme is cute\n".chars().map(|x| x.to_string()).collect::<Vec<String>>().join("\n"))
            .assert()
            .code(0)
            .stdout("pomme is cute\n");
    }

    #[test]
    fn input_byte_as_number_mode(){
        let mut cmd = Command::cargo_bin("brainpurr").unwrap();
        let _ = cmd
            .args(&["./examples/echo.bp", "--input", "byte-as-number"])
            .write_stdin("pomme is cute\n".chars().map(|x| format!("{}\n", x as u8)).collect::<String>())
            .assert()
            .code(0)
            .stdout("pomme is cute\n");
    }

    #[test]
    fn output_normal_mode(){
        let instructions = vec![Instruction::ByteIncrement(67), Instruction::ByteOutput];
        let mut result = vec![];
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false, None);
        interpreter(instructions, &args, &mut result);
        assert_eq!(result, vec![67])
    }

    #[test]
    fn output_byte_as_number(){
        let instructions = vec![Instruction::ByteIncrement(67), Instruction::ByteOutput];
        let mut result = vec![];
        let args = Args::new(InputMethod::Normal, OutputMethod::ByteAsNumber, false, None);
        interpreter(instructions, &args, &mut result);
        assert_eq!(result, vec!['6' as u8, '7' as u8, '\n' as u8])
    }

    #[test]
    fn nya_loop(){
        let mut instructions = vec![
            Instruction::ByteIncrement(7),
            Instruction::OpenLoop(6),
            Instruction::ByteDecrement(1),
            Instruction::PointerIncrement(1),
            Instruction::ByteIncrement(6),
            Instruction::PointerDecrement(1),
            Instruction::CloseLoop(1),
        ];
        instructions.push(Instruction::ByteOutput);
        let args = Args::new(InputMethod::Normal, OutputMethod::ByteAsNumber, false, None);
        let result = interpreter(instructions, &args, &mut vec![]);
        assert_eq!(result, vec![0, 42])
    }

    #[test]
    fn newline_zero(){
        // normal mode for both
        let mut cmd = Command::cargo_bin("brainpurr").unwrap();
        cmd.args(&["./examples/tests/newline_zero.bp", "--newline-zero"])
            .write_stdin("\n\0")
            .assert()
            .code(0)
            .stdout("\n\0\n\0");
        // byte-as-number for both
        let mut cmd = Command::cargo_bin("brainpurr").unwrap();
        cmd
            .args(&["./examples/tests/newline_zero.bp", "--newline-zero", "--input", "byte-as-number", "--output", "byte-as-number"])
            .write_stdin("10\n0\n")
            .assert()
            .code(0)
            .stdout("10\n0\n10\n0\n");
        // first-char-only mode for input
        let mut cmd = Command::cargo_bin("brainpurr").unwrap();
        cmd.args(&["./examples/tests/newline_zero.bp", "--newline-zero", "--input", "first-char-only"])
            .write_stdin("\n\0\n")
            .assert()
            .code(0)
            .stdout("\n\0\n\0");
    }

    #[test]
    fn max_array_size_border(){
        let mut cmd = Command::cargo_bin("brainpurr").unwrap();
        cmd.args(&["./examples/tests/max_array_size_border.bp", "--max-array-size", "10"])
            .write_stdin("")
            .assert()
            .code(0)
            .stdout("");
    }
    #[test]
    fn max_array_size_overflow(){
        let mut cmd = Command::cargo_bin("brainpurr").unwrap();
        cmd.args(&["./examples/tests/max_array_size_border.bp", "--max-array-size", "9"])
            .write_stdin("")
            .assert()
            .stdout("")
            .code(1)
            .failure();
    }
}
