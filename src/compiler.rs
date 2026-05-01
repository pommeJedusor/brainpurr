use std::{fs::{self, File}, hash::{DefaultHasher, Hash, Hasher}, io::Write, ops::Add, process::{Command}};

use crate::{InputMethod, OutputMethod, parser::Instruction};

pub fn compile_to_c(instructions: &Vec<Instruction>, max_array_size: Option<u32>, input_method: &InputMethod, output_method: &OutputMethod) -> String {
    let max_array_size = max_array_size.unwrap_or(67000);

    let c_file = "#include <stdio.h>\n".to_string();
    let c_file = c_file.add(&format!("const unsigned long ARRAY_LENGTH = {};\n", max_array_size));
    // TODO make the input array size customizable
    let c_file = c_file.add("int main(){\nchar array[ARRAY_LENGTH];\nchar input[100];\nfor (int i=0;i<ARRAY_LENGTH;i++){\narray[i] = 0;\n}\nunsigned long pointer = 0;\n");

    // TODO stderr, stdout handling
    let c_file = c_file.add(&instructions.iter().map(|x| match x{
        Instruction::PointerIncrement => "if (pointer >= ARRAY_LENGTH - 1){printf(\"pointer overflow\\n\");return 1;}pointer++;",
        Instruction::PointerDecrement => "if (pointer == 0){printf(\"pointer underflow\");return 1;}pointer--;",
        Instruction::ByteIncrement => "array[pointer]++;",
        Instruction::ByteDecrement => "array[pointer]--;",
        Instruction::ByteInput => match input_method{
            InputMethod::Normal => "scanf(\"%c\", &array[pointer]);",
            InputMethod::FirstCharOnly => "fgets(input, 100, stdin);array[pointer] = input[0];",
            InputMethod::ByteAsNumber => "scanf(\"%d\", &array[pointer]);",
        },
        Instruction::ByteOutput => match output_method {
            OutputMethod::Normal => "printf(\"%c\", array[pointer]);",
            OutputMethod::ByteAsNumber => "printf(\"%d\\n\", array[pointer]);",
        }
        Instruction::OpenLoop(_) => "while (array[pointer] != 0){",
        Instruction::CloseLoop(_) => "}",
    }).collect::<Vec<&str>>().join("\n"));

    let c_file = c_file.add("return 0;}");

    c_file
}

pub fn compile_to_file(instructions: &Vec<Instruction>, max_array_size: Option<u32>, input_method: &InputMethod, output_method: &OutputMethod, gcc_args: &Vec<&str>){
    let c_code = compile_to_c(instructions, max_array_size, input_method, output_method);

    let mut hasher = DefaultHasher::new();
    (instructions, max_array_size, input_method, output_method, gcc_args).hash(&mut hasher);
    let c_file_name = format!("temp-{}.c", hasher.finish());

    let mut gcc_args = gcc_args.clone();
    gcc_args.insert(0, &c_file_name);
    println!("{:?}", gcc_args);

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

    fn expect_success(exe_name: &str, args: &Vec<&str>, expected_output: &str, stdin: &str){
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
    fn expect_compiler_failure(args: &Vec<&str>){
        let mut cmd = Command::cargo_bin("brainpurr").unwrap();
        cmd
            .args(args)
            .assert()
            .failure();
    }
    fn expect_binary_failure(exe_name: &str, args: &Vec<&str>, stdin: &str){
        let mut cmd = Command::cargo_bin("brainpurr").unwrap();
        cmd
            .args(args)
            .assert()
            .code(0);
        let cmd = Command::new(exe_name).write_stdin(stdin).assert();
        cmd
            .code(1)
            .failure();
        fs::remove_file(exe_name).unwrap();
    }

    #[test]
    fn increment(){
        expect_success("./increment.out", &vec!["./examples/tests/increment.bp", "--compile", "--gcc-args", "-o increment.out"], "C", "");
    }
    #[test]
    fn increment_overflow(){
        expect_success("./increment_overflow.out", &vec!["./examples/tests/increment_overflow.bp", "--compile", "--gcc-args", "-o increment_overflow.out"], "C", "");
    }

    #[test]
    fn decrement(){
        expect_success("./decrement.out", &vec!["./examples/tests/decrement.bp", "--compile", "--gcc-args", "-o decrement.out"], "C", "");
    }
    #[test]
    fn decrement_overflow(){
        expect_success("./decrement_overflow.out", &vec!["./examples/tests/decrement_overflow.bp", "--compile", "--gcc-args", "-o decrement_overflow.out"], "C", "");
    }

    #[test]
    fn pointer_increment(){
        expect_success("./pointer_increment.out", &vec!["./examples/tests/pointer_increment.bp", "--compile", "--gcc-args", "-o pointer_increment.out"], "C\n", "");
    }
    #[test]
    fn pointer_increment_overflow(){
        expect_binary_failure("./pointer_increment_overflow.out", &vec!["./examples/tests/pointer_increment_overflow.bp", "--compile", "--gcc-args", "-o pointer_increment_overflow.out"], "");
    }

    #[test]
    fn pointer_decrement(){
        expect_success("./pointer_decrement.out", &vec!["./examples/tests/pointer_decrement.bp", "--compile", "--gcc-args", "-o pointer_decrement.out"], "C\n", "");
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
        expect_success("./input_first_char_only.out", &vec!["./examples/echo.bp", "--input", "first-char-only", "--compile", "--gcc-args", "-o input_first_char_only.out"], "p\n", &"p\n".chars().map(|x| x.to_string()).collect::<Vec<String>>().join("\n"));
    }
    #[test]
    fn input_byte_as_number(){
        expect_success("./input_byte_as_number.out", &vec!["./examples/echo.bp", "--input", "byte-as-number", "--compile", "--gcc-args", "-o input_byte_as_number.out"], "pomme is cute\n", &"pomme is cute\n".chars().map(|x| format!("{}\n", x as u8)).collect::<String>());
    }

    #[test]
    fn output(){
        expect_success("./output.out", &vec!["./examples/tests/output.bp", "--compile", "--gcc-args", "-o output.out"], "\0", "");
    }
    #[test]
    fn output_normal_mode(){
        expect_success("./output_normal_mode.out", &vec!["./examples/echo.bp", "--output", "normal", "--compile", "--gcc-args", "-o output_normal_mode.out"], "pomme is cute\n", "pomme is cute\n");
    }
    #[test]
    fn output_byte_as_number(){
        expect_success("./output_byte_as_number.out", &vec!["./examples/echo.bp", "--output", "byte-as-number", "--compile", "--gcc-args", "-o output_byte_as_number.out"], "67\n10\n", "C\n");
    }

    // TODO loops, max array size, input method, output method
}
