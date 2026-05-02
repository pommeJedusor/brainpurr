use std::io::{self, Write};

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

// TODO implement max_array_size
pub fn interpreter<T: InterpreterArgs>(instructions: Vec<Instruction>, args: &T, mut writer: impl std::io::Write) -> Vec<u8> {
    let input_method = args.get_input_method();
    let output_method = args.get_output_method();
    let newline_zero = args.get_newline_zero();
    
    let newline_zero_func = if newline_zero {newline_zero_map} else {useless_map};

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
    }

    impl InterpreterArgs for Args {
        fn get_input_method(&self) -> &InputMethod { &self.input }
        fn get_output_method(&self) -> &OutputMethod { &self.output }
        fn get_newline_zero(&self) -> bool { self.newline_zero }
    }

    impl Args {
        fn new(input: InputMethod, output: OutputMethod, newline_zero: bool) -> Self{
            Self { input, output, newline_zero }
        }
    }

    #[test]
    fn increment(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false);
        assert_eq!(interpreter(vec![Instruction::ByteIncrement], &args, &mut vec![]), vec![1]);
    }

    #[test]
    fn increment_overflow(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false);
        assert_eq!(interpreter(vec![Instruction::ByteIncrement; 256], &args, &mut vec![]), vec![0]);
    }

    #[test]
    fn decrement(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false);
        assert_eq!(interpreter(vec![Instruction::ByteIncrement, Instruction::ByteDecrement], &args, &mut vec![]), vec![0]);
    }

    #[test]
    fn decrement_overflow(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false);
        assert_eq!(interpreter(vec![Instruction::ByteDecrement; 256], &args, &mut vec![]), vec![0]);
    }

    #[test]
    fn pointer_increment(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false);
        assert_eq!(interpreter(vec![Instruction::PointerIncrement, Instruction::ByteIncrement], &args, &mut vec![]), vec![0, 1]);
    }

    #[test]
    fn pointer_decrement(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false);
        assert_eq!(interpreter(vec![Instruction::PointerIncrement, Instruction::PointerDecrement, Instruction::ByteIncrement], &args, &mut vec![]), vec![1, 0]);
    }

    #[test]
    #[should_panic]
    fn pointer_decrement_overflow(){
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false);
        interpreter(vec![Instruction::PointerDecrement], &args, &mut vec![]);
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
        let mut instructions = vec![Instruction::ByteIncrement; 67];
        instructions.push(Instruction::ByteOutput);
        let mut result = vec![];
        let args = Args::new(InputMethod::Normal, OutputMethod::Normal, false);
        interpreter(instructions, &args, &mut result);
        assert_eq!(result, vec![67])
    }

    #[test]
    fn output_byte_as_number(){
        let mut instructions = vec![Instruction::ByteIncrement; 67];
        instructions.push(Instruction::ByteOutput);
        let mut result = vec![];
        let args = Args::new(InputMethod::Normal, OutputMethod::ByteAsNumber, false);
        interpreter(instructions, &args, &mut result);
        assert_eq!(result, vec!['6' as u8, '7' as u8, '\n' as u8])
    }

    #[test]
    fn nya_loop(){
        let mut instructions = vec![
            Instruction::ByteIncrement,
            Instruction::ByteIncrement,
            Instruction::ByteIncrement,
            Instruction::ByteIncrement,
            Instruction::ByteIncrement,
            Instruction::ByteIncrement,
            Instruction::ByteIncrement,
            Instruction::OpenLoop(16),
            Instruction::ByteDecrement,
            Instruction::PointerIncrement,
            Instruction::ByteIncrement,
            Instruction::ByteIncrement,
            Instruction::ByteIncrement,
            Instruction::ByteIncrement,
            Instruction::ByteIncrement,
            Instruction::ByteIncrement,
            Instruction::PointerDecrement,
            Instruction::CloseLoop(7),
        ];
        instructions.push(Instruction::ByteOutput);
        let args = Args::new(InputMethod::Normal, OutputMethod::ByteAsNumber, false);
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
        cmd.args(&["./examples/tests/newline_zero.bp", "--newline-zero"])
            .write_stdin("\n\0\n")
            .assert()
            .code(0)
            .stdout("\n\0\n\0");
    }
}
