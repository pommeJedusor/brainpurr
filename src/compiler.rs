use std::{fs::{self, File}, hash::{DefaultHasher, Hash, Hasher}, io::Write, ops::Add, process::{Command}};

use crate::{InputMethod, OutputMethod, args::CompilerArgs, parser::Instruction};

struct Instructions{
    pointer_increment: String,
    pointer_decrement: String,
    byte_increment: String,
    byte_decrement: String,
    byte_input: String,
    byte_output: String,
    open_loop: String,
    close_loop: String,
}

impl Instructions{
    fn get_c_instructions<T: CompilerArgs>(args: &T) -> Self{
        let input_method = args.get_input_method();
        let output_method = args.get_output_method();
        let newline_zero = args.get_newline_zero();
        let newline_zero_input_instruction = match newline_zero{
            true => "if (array[pointer] == 10){\narray[pointer] = 0;\n}else if (array[pointer] == 0){\narray[pointer] = 10;\n}",
            false => "",
        };

        let pointer_increment = "if (pointer >= ARRAY_LENGTH - 1){\nfprintf(stderr, \"pointer overflow\\n\");\nreturn 1;\n}\npointer++;".to_string();
        let pointer_decrement = "if (pointer == 0){\nfprintf(stderr, \"pointer underflow\");\nreturn 1;\n}\npointer--;".to_string();
        let byte_increment = "array[pointer]++;".to_string();
        let byte_decrement = "array[pointer]--;".to_string();
        let byte_input = match input_method{
                InputMethod::Normal => format!("scanf(\"%c\", &array[pointer]);\n{}", newline_zero_input_instruction),
                InputMethod::FirstCharOnly => format!("while (1){{\nfgets(input, INPUT_LENGTH, stdin);\nif (is_first_char_found == 0){{\nfirst_char = input[0];\nis_first_char_found = 1;\n}}\nif (input[0] == 10){{\nbreak;\n}}\n}}\narray[pointer] = first_char;\nis_first_char_found = 0;\n{}", newline_zero_input_instruction),
                InputMethod::ByteAsNumber => format!("scanf(\"%d\", &array[pointer]);\n{}", newline_zero_input_instruction),
            };
        let byte_output = match output_method {
            OutputMethod::Normal => match newline_zero {
                true => "if (array[pointer] == 10){\nprintf(\"%c\", 0);\n}else if (array[pointer] == 0){\nprintf(\"%c\", 10);\n}else {\nprintf(\"%c\", array[pointer]);\n}".to_string(),
                false => "printf(\"%c\", array[pointer]);".to_string(),
            },
            OutputMethod::ByteAsNumber =>  match newline_zero {
                true => "if (array[pointer] == 10){\nprintf(\"%d\\n\", 0);\n}else if (array[pointer] == 0){\nprintf(\"%d\\n\", 10);\n}else {\nprintf(\"%d\\n\", array[pointer]);\n}".to_string(),
                false => "printf(\"%d\\n\", array[pointer]);".to_string(),
            }
        };
        let open_loop = "while (array[pointer] != 0){".to_string();
        let close_loop = "}".to_string();
        Self { pointer_increment, pointer_decrement, byte_increment, byte_decrement, byte_input, byte_output, open_loop, close_loop }
    }
}

pub fn compile_to_c<T: CompilerArgs>(instructions: &Vec<Instruction>, args: &T) -> String {
    let max_array_size = args.get_max_array_size();
    let c_instructions = Instructions::get_c_instructions(args);

    let c_file = "#include <stdio.h>\n".to_string();
    let c_file = c_file.add(&format!("const unsigned long ARRAY_LENGTH = {};\nconst int INPUT_LENGTH = 2;\n", max_array_size));
    let c_file = c_file.add("int main(){\nchar array[ARRAY_LENGTH];\nchar input[INPUT_LENGTH];\nchar first_char = 0;\nchar is_first_char_found = 0;\nfor (int i=0;i<ARRAY_LENGTH;i++){\narray[i] = 0;\n}\nunsigned long pointer = 0;\n");

    let c_file = c_file.add(&instructions.iter().map(|x| match x{
        Instruction::PointerIncrement => &c_instructions.pointer_increment,
        Instruction::PointerDecrement => &c_instructions.pointer_decrement,
        Instruction::ByteIncrement => &c_instructions.byte_increment,
        Instruction::ByteDecrement => &c_instructions.byte_decrement,
        Instruction::ByteInput => &c_instructions.byte_input,
        Instruction::ByteOutput => &c_instructions.byte_output,
        Instruction::OpenLoop(_) => &c_instructions.open_loop,
        Instruction::CloseLoop(_) => &c_instructions.close_loop,
    }).map(|x| x as &str).collect::<Vec<&str>>().join("\n"));

    let c_file = c_file.add("\nreturn 0;\n}");

    c_file
}

pub fn compile_to_file<T: CompilerArgs>(instructions: &Vec<Instruction>, args: &T){
    let max_array_size = args.get_max_array_size();
    let input_method = args.get_input_method();
    let output_method = args.get_output_method();
    let gcc_args = args.get_gcc_args().unwrap_or("".to_string());
    let gcc_args = gcc_args.split(" ").filter(|x| x != &"").collect::<Vec<&str>>();

    let c_code = compile_to_c(instructions, args);

    let mut hasher = DefaultHasher::new();
    (instructions, max_array_size, input_method, output_method, &gcc_args).hash(&mut hasher);
    let c_file_name = format!("temp-{}.c", hasher.finish());

    let mut gcc_args = gcc_args.clone();
    gcc_args.insert(0, &c_file_name);

    let mut file = File::create(&c_file_name).expect("failed to create temporary file for compiling");
    write!(file, "{}", c_code).unwrap();
    let result = Command::new("gcc")
        .args(&gcc_args)
        .status()
        .expect("failed to compile the code using gcc (gcc is required to run this command)");
    assert!(result.success());

    fs::remove_file(&c_file_name).expect("failed to delete temporary file for compiling");
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::{Command, assert::OutputAssertExt};
    use predicates::prelude::predicate;

    fn expect_success(exe_name: &str, args: &Vec<&str>, stdin: &str, expected_output: &str){
        let mut cmd = Command::cargo_bin("brainpurr").unwrap();
        cmd
            .args(args)
            .assert()
            .code(0);
        let cmd = Command::new(exe_name).write_stdin(stdin).unwrap();
        let _ = cmd
            .assert()
            .code(0)
            .stdout(predicate::eq(expected_output.to_string().into_bytes()));
        fs::remove_file(exe_name).unwrap();
    }
    fn expect_binary_failure(exe_name: &str, args: &Vec<&str>, stdin: &str){
        let mut cmd = Command::cargo_bin("brainpurr").unwrap();
        cmd
            .args(args)
            .assert()
            .code(0);
        let cmd = Command::new(exe_name).write_stdin(stdin).assert();
        cmd.code(1).failure().stdout(predicate::str::is_empty());
        fs::remove_file(exe_name).unwrap();
    }

    #[test]
    fn increment(){
        expect_success("./increment.out", &vec!["./examples/tests/increment.bp", "--compile", "--gcc-args", "-o increment.out"], "", "C");
    }
    #[test]
    fn increment_overflow(){
        expect_success("./increment_overflow.out", &vec!["./examples/tests/increment_overflow.bp", "--compile", "--gcc-args", "-o increment_overflow.out"], "", "C");
    }

    #[test]
    fn decrement(){
        expect_success("./decrement.out", &vec!["./examples/tests/decrement.bp", "--compile", "--gcc-args", "-o decrement.out"], "", "C");
    }
    #[test]
    fn decrement_overflow(){
        expect_success("./decrement_overflow.out", &vec!["./examples/tests/decrement_overflow.bp", "--compile", "--gcc-args", "-o decrement_overflow.out"], "", "C");
    }

    #[test]
    fn pointer_increment(){
        expect_success("./pointer_increment.out", &vec!["./examples/tests/pointer_increment.bp", "--compile", "--gcc-args", "-o pointer_increment.out"], "", "C\n");
    }
    #[test]
    fn pointer_increment_overflow(){
        expect_binary_failure("./pointer_increment_overflow.out", &vec!["./examples/tests/pointer_increment_overflow.bp", "--compile", "--gcc-args", "-o pointer_increment_overflow.out"], "");
    }

    #[test]
    fn pointer_decrement(){
        expect_success("./pointer_decrement.out", &vec!["./examples/tests/pointer_decrement.bp", "--compile", "--gcc-args", "-o pointer_decrement.out"], "", "C\n");
    }
    #[test]
    fn pointer_decrement_overflow(){
        expect_binary_failure("./pointer_decrement_overflow.out", &vec!["./examples/tests/pointer_decrement_overflow.bp", "--compile", "--gcc-args", "-o pointer_decrement_overflow.out"], "");
    }

    #[test]
    fn input(){
        expect_success("./input.out", &vec!["./examples/tests/input.bp", "--compile", "--gcc-args", "-o input.out"], "C", "C");
    }
    #[test]
    fn input_normal_mode(){
        expect_success("./input_normal_mode.out", &vec!["./examples/echo.bp", "--compile", "--gcc-args", "-o input_normal_mode.out"], "pomme is cute\n", "pomme is cute\n");
    }
    #[test]
    fn input_first_char_only(){
        expect_success("./input_first_char_only.out", &vec!["./examples/echo.bp", "--input", "first-char-only", "--compile", "--gcc-args", "-o input_first_char_only.out"], &"p\n".chars().map(|x| x.to_string()).collect::<Vec<String>>().join("\n"), "p\n");
    }
    #[test]
    fn input_byte_as_number(){
        expect_success("./input_byte_as_number.out", &vec!["./examples/echo.bp", "--input", "byte-as-number", "--compile", "--gcc-args", "-o input_byte_as_number.out"], &"pomme is cute\n".chars().map(|x| format!("{}\n", x as u8)).collect::<String>(), "pomme is cute\n");
    }

    #[test]
    fn output(){
        expect_success("./output.out", &vec!["./examples/tests/output.bp", "--compile", "--gcc-args", "-o output.out"], "", "\0");
    }
    #[test]
    fn output_normal_mode(){
        expect_success("./output_normal_mode.out", &vec!["./examples/echo.bp", "--output", "normal", "--compile", "--gcc-args", "-o output_normal_mode.out"], "pomme is cute\n", "pomme is cute\n");
    }
    #[test]
    fn output_byte_as_number(){
        expect_success("./output_byte_as_number.out", &vec!["./examples/echo.bp", "--output", "byte-as-number", "--compile", "--gcc-args", "-o output_byte_as_number.out"], "C\n", "67\n10\n");
    }

    #[test]
    fn useless_loop(){
        expect_success("./useless_loop.out", &vec!["./examples/tests/useless_loop.bp", "--compile", "--gcc-args", "-o useless_loop.out"], "", "\0");
    }
    #[test]
    fn useful_loop(){
        expect_success("./useful_loop.out", &vec!["./examples/tests/useful_loop.bp", "--compile", "--gcc-args", "-o useful_loop.out"], "", "\n");
    }

    #[test]
    fn max_array_size_border(){
        expect_success("./max_array_size_border.out", &vec!["./examples/tests/max_array_size_border.bp", "--compile", "--gcc-args", "-o max_array_size_border.out", "--max-array-size", "10"], "", "");
    }
    #[test]
    fn max_array_size_overflow(){
        expect_binary_failure("./max_array_size_overflow.out", &vec!["./examples/tests/max_array_size_border.bp", "--compile", "--gcc-args", "-o max_array_size_overflow.out", "--max-array-size", "9"], "");
    }
    #[test]
    fn array_empty(){
        expect_binary_failure("./array_empty.out", &vec!["./examples/tests/array_empty.bp", "--compile", "--gcc-args", "-o array_empty.out", "--max-array-size", "1000000"], "");
    }

    #[test]
    fn newline_zero(){
        //// normal mode for both
        expect_success("./newline_zero_normal_mode.out", &vec!["./examples/tests/newline_zero.bp", "--compile", "--gcc-args", "-o newline_zero_normal_mode.out", "--newline-zero", "--input", "normal", "--output", "normal"], "\n\0", "\n\0\n\0");
        //// byte-as-number for both
        expect_success("./newline_zero_byte_as_number.out", &vec!["./examples/tests/newline_zero.bp", "--compile", "--gcc-args", "-o newline_zero_byte_as_number.out", "--newline-zero", "--input", "byte-as-number", "--output", "byte-as-number"], "10\n0\n", "10\n0\n10\n0\n");
        //// first-char-only mode for input
        expect_success("./newline_zero_first_char_only.out", &vec!["./examples/tests/newline_zero.bp", "--compile", "--gcc-args", "-o newline_zero_first_char_only.out", "--newline-zero", "--input", "first-char-only"], "\n\0\n", "\n\0\n\0");
    }

    #[test]
    fn input_length_overflow(){
        expect_success("./input_length_overflow.out", &vec!["./examples/echo.bp", "--input", "first-char-only", "--compile", "--gcc-args", "-o input_length_overflow.out"], "ppppppppp\no\n\0\n\n", "po\0\n");
    }
}
