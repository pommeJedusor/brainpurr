use std::{
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    ops::Add,
    process::Command,
};

use crate::{
    args::{CompilerArgs, InputMethod, OutputMethod, PointerWrapMode},
    error::BrainpurrError,
    parser::Instruction,
};

fn get_pointer_increment_instruction(
    pointer_wrap_mode: &PointerWrapMode,
    x: usize,
    array_length: u32,
) -> String {
    match pointer_wrap_mode {
        PointerWrapMode::Crash => format!(
            "if (pointer >= ARRAY_LENGTH - {}){{\nfprintf(stderr, \"pointer overflow\\n\");\nreturn 1;\n}}\npointer += {};",
            x, x
        ),
        PointerWrapMode::Unsafe => format!("pointer += {};", x),
        PointerWrapMode::WrapAround => format!("pointer = (pointer + {}) % {};", x, array_length),
        PointerWrapMode::Stick => format!(
            "if (pointer + {} >= ARRAY_LENGTH - 1){{\npointer = ARRAY_LENGTH - 1;\n}}\nelse{{\npointer += {};\n}}",
            x, x
        ),
    }
}

fn get_pointer_decrement_instruction(
    pointer_wrap_mode: &PointerWrapMode,
    x: usize,
    array_length: u32,
) -> String {
    match pointer_wrap_mode {
        PointerWrapMode::Crash => format!(
            "if (pointer < {x}){{\nfprintf(stderr, \"pointer underflow\\n\");\nreturn 1;\n}}\npointer -= {x};"
        ),
        PointerWrapMode::Unsafe => format!("pointer -= {x};"),
        PointerWrapMode::WrapAround => format!(
            "pointer = (pointer + {array_length} - ({x} % {array_length})) % {array_length};"
        ),
        PointerWrapMode::Stick => {
            format!("if (pointer < {x}){{\npointer = 0;\n}}\nelse{{\npointer -= {x};\n}}")
        }
    }
}

fn get_byte_increment_instruction(x: usize) -> String {
    format!("array[pointer] += {};", x)
}

fn get_byte_decrement_instruction(x: usize) -> String {
    format!("array[pointer] -= {};", x)
}

fn get_input_instruction(input_method: &InputMethod, newline_zero: bool) -> String {
    let newline_zero_input_instruction = match newline_zero {
        true => {
            "if (array[pointer] == 10){\narray[pointer] = 0;\n}else if (array[pointer] == 0){\narray[pointer] = 10;\n}"
        }
        false => "",
    };

    match input_method {
        InputMethod::Normal => format!(
            "scanf(\"%c\", &array[pointer]);\n{}",
            newline_zero_input_instruction
        ),
        InputMethod::FirstCharOnly => format!(
            "while (1){{\nfgets(input, INPUT_LENGTH, stdin);\nif (is_first_char_found == 0){{\nfirst_char = input[0];\nis_first_char_found = 1;\n}}\nif (input[0] == 10){{\nbreak;\n}}\n}}\narray[pointer] = first_char;\nis_first_char_found = 0;\n{}",
            newline_zero_input_instruction
        ),
        InputMethod::ByteAsNumber => format!(
            "scanf(\"%d\", &array[pointer]);\n{}",
            newline_zero_input_instruction
        ),
    }
}

fn get_output_instruction(output_method: &OutputMethod, newline_zero: bool) -> String {
    match output_method {
        OutputMethod::Normal => match newline_zero {
            true => "if (array[pointer] == 10){\nprintf(\"%c\", 0);\n}else if (array[pointer] == 0){\nprintf(\"%c\", 10);\n}else {\nprintf(\"%c\", array[pointer]);\n}".to_string(),
            false => "printf(\"%c\", array[pointer]);".to_string(),
        },
        OutputMethod::ByteAsNumber =>  match newline_zero {
            true => "if (array[pointer] == 10){\nprintf(\"%d\\n\", 0);\n}else if (array[pointer] == 0){\nprintf(\"%d\\n\", 10);\n}else {\nprintf(\"%d\\n\", array[pointer]);\n}".to_string(),
            false => "printf(\"%d\\n\", array[pointer]);".to_string(),
        }
    }
}

pub fn compile_to_c<T: CompilerArgs>(instructions: &[Instruction], args: &T) -> String {
    let max_array_size = args.get_max_array_size();
    let pointer_wrap_mode = args.get_pointer_wrap_mode();

    let c_file = "#include <stdio.h>\n".to_string();
    let c_file = c_file.add(&format!(
        "const unsigned long ARRAY_LENGTH = {};\nconst int INPUT_LENGTH = 2;\n",
        max_array_size
    ));
    let c_file = c_file.add("int main(){\nchar array[ARRAY_LENGTH];\nchar input[INPUT_LENGTH];\nchar first_char = 0;\nchar is_first_char_found = 0;\nfor (int i=0;i<ARRAY_LENGTH;i++){\narray[i] = 0;\n}\nunsigned long pointer = 0;\n");

    let c_file = c_file.add(
        &instructions
            .iter()
            .map(|x| match x {
                Instruction::PointerIncrement(x) => {
                    get_pointer_increment_instruction(pointer_wrap_mode, *x, max_array_size)
                }
                Instruction::PointerDecrement(x) => {
                    get_pointer_decrement_instruction(pointer_wrap_mode, *x, max_array_size)
                }
                Instruction::ByteIncrement(x) => get_byte_increment_instruction(*x),
                Instruction::ByteDecrement(x) => get_byte_decrement_instruction(*x),
                Instruction::ByteInput => {
                    get_input_instruction(args.get_input_method(), args.get_newline_zero())
                }
                Instruction::ByteOutput => {
                    get_output_instruction(args.get_output_method(), args.get_newline_zero())
                }
                Instruction::OpenLoop(_) => "while (array[pointer] != 0){".to_string(),
                Instruction::CloseLoop(_) => "}".to_string(),
            })
            .collect::<Vec<String>>()
            .join("\n"),
    );

    c_file.add("\nreturn 0;\n}")
}

pub fn compile_to_file<T: CompilerArgs>(
    instructions: &Vec<Instruction>,
    args: &T,
) -> Result<(), BrainpurrError> {
    let max_array_size = args.get_max_array_size();
    let input_method = args.get_input_method();
    let output_method = args.get_output_method();
    let gcc_args = args.get_gcc_args().unwrap_or("".to_string());
    let mut gcc_args = gcc_args
        .split(" ")
        .filter(|x| x != &"")
        .collect::<Vec<&str>>();

    let c_code = compile_to_c(instructions, args);

    let mut hasher = DefaultHasher::new();
    (
        instructions,
        max_array_size,
        input_method,
        output_method,
        &gcc_args,
    )
        .hash(&mut hasher);
    let c_file_name = format!("temp-{}.c", hasher.finish());

    gcc_args.insert(0, &c_file_name);

    let mut file = match File::create(&c_file_name) {
        Ok(file) => file,
        Err(err) => Err(BrainpurrError::CompilingError(format!(
            "failed to create temporary file for compiling: {err}"
        )))?,
    };

    if let Err(err) = write!(file, "{}", c_code) {
        return Err(BrainpurrError::CompilingError(err.to_string()));
    }
    let result = match Command::new("gcc").args(&gcc_args).status() {
        Ok(result) => result,
        Err(err) => Err(BrainpurrError::CompilingError(err.to_string()))?,
    };
    assert!(result.success());

    if let Err(err) = fs::remove_file(&c_file_name) {
        return Err(BrainpurrError::CompilingError(format!(
            "failed to delete temporary file for compiling: {err}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;

    fn compile(exe_name: &str, args: &[&str]) {
        let mut args = args.to_owned();
        let gcc_args = &format!("-o {exe_name}");
        args.push("--compile");
        args.push("--gcc-args");
        args.push(gcc_args);
        let mut cmd = Command::cargo_bin("brainpurr").unwrap();
        cmd.args(args).assert().code(0);
    }

    // mrp
    #[test]
    fn increment() {
        let exe_name = "./increment.out";
        compile(exe_name, &["./examples/tests/increment.bp"]);
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("C");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn byte_overflow() {
        let exe_name = "./byte_overflow.out";
        compile(exe_name, &["./examples/tests/byte_overflow.bp"]);
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("C");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn increment_overflow() {
        let exe_name = "./increment_overflow.out";
        compile(
            exe_name,
            &[
                "./examples/tests/increment_overflow.bp",
                "--output",
                "byte-as-number",
            ],
        );
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("0\n");
        fs::remove_file(exe_name).unwrap();
    }

    // purr
    #[test]
    fn decrement() {
        let exe_name = "./decrement.out";
        compile(exe_name, &["./examples/tests/decrement.bp"]);
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("C");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn byte_underflow() {
        let exe_name = "./byte_underflow.out";
        compile(exe_name, &["./examples/tests/byte_underflow.bp"]);
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("C");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn decrement_overflow() {
        let exe_name = "./decrement_overflow.out";
        compile(
            exe_name,
            &[
                "./examples/tests/decrement_overflow.bp",
                "--output",
                "byte-as-number",
            ],
        );
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("0\n");
        fs::remove_file(exe_name).unwrap();
    }

    // meow
    #[test]
    fn pointer_increment() {
        let exe_name = "./pointer_increment.out";
        compile(exe_name, &["./examples/tests/pointer_increment.bp"]);
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("C\n");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn pointer_overflow() {
        let exe_name = "./pointer_overflow.out";
        compile(exe_name, &["./examples/tests/pointer_overflow.bp"]);
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .failure()
            .code(1)
            .stdout("");
        fs::remove_file(exe_name).unwrap();
    }
    // PointerWrapMode
    #[test]
    fn pointer_increment_overflow_wrap_around() {
        let exe_name = "./pointer_increment_overflow_wrap_around.out";
        compile(
            exe_name,
            &[
                "./examples/tests/pointer_increment_overflow_wrap_around.bp",
                "--pointer-wrap",
                "wrap-around",
                "--max-array-size",
                "2",
            ],
        );
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("\0\n");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn pointer_increment_overflow_wrap_around_stick() {
        let exe_name = "./pointer_increment_overflow_wrap_around_stick.out";
        compile(
            exe_name,
            &[
                "./examples/tests/pointer_increment_overflow_wrap_around.bp",
                "--pointer-wrap",
                "stick",
                "--max-array-size",
                "2",
            ],
        );
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("\0\0");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn pointer_increment_overflow_wrap_around_crash() {
        let exe_name = "./pointer_increment_overflow_wrap_around_crash.out";
        compile(
            exe_name,
            &[
                "./examples/tests/pointer_increment_overflow_wrap_around.bp",
                "--pointer-wrap",
                "crash",
                "--max-array-size",
                "2",
            ],
        );
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .failure()
            .code(1)
            .stdout("\0");
        fs::remove_file(exe_name).unwrap();
    }

    // mrow
    #[test]
    fn pointer_decrement() {
        let exe_name = "./pointer_decrement.out";
        compile(exe_name, &["./examples/tests/pointer_decrement.bp"]);
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("C\n");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn pointer_underflow() {
        let exe_name = "./pointer_underflow.out";
        compile(exe_name, &["./examples/tests/pointer_underflow.bp"]);
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .failure()
            .code(1)
            .stdout("");
        fs::remove_file(exe_name).unwrap();
    }
    // PointerWrapMode
    #[test]
    fn pointer_decrement_underflow_wrap_around() {
        let exe_name = "./pointer_decrement_underflow_wrap_around.out";
        compile(
            exe_name,
            &[
                "./examples/tests/pointer_decrement_underflow_wrap_around.bp",
                "--pointer-wrap",
                "wrap-around",
                "--max-array-size",
                "2",
            ],
        );
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("\0\n");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn pointer_decrement_underflow_wrap_around_stick() {
        let exe_name = "./pointer_decrement_underflow_wrap_around_stick.out";
        compile(
            exe_name,
            &[
                "./examples/tests/pointer_decrement_underflow_wrap_around.bp",
                "--pointer-wrap",
                "stick",
                "--max-array-size",
                "2",
            ],
        );
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("\0\0");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn pointer_decrement_underflow_wrap_around_crash() {
        let exe_name = "./pointer_decrement_underflow_wrap_around_crash.out";
        compile(
            exe_name,
            &[
                "./examples/tests/pointer_decrement_underflow_wrap_around.bp",
                "--pointer-wrap",
                "crash",
                "--max-array-size",
                "2",
            ],
        );
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .failure()
            .code(1)
            .stdout("\0");
        fs::remove_file(exe_name).unwrap();
    }

    // >:3
    #[test]
    fn input() {
        let exe_name = "./input.out";
        compile(exe_name, &["./examples/tests/input.bp"]);
        Command::new(exe_name)
            .write_stdin("C")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("C");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn input_normal_mode() {
        let exe_name = "./input_normal_mode.out";
        compile(exe_name, &["./examples/echo.bp"]);
        Command::new(exe_name)
            .write_stdin("pomme is cute\n")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("pomme is cute\n");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn input_first_char_only() {
        let exe_name = "./input_first_char_only.out";
        compile(
            exe_name,
            &["./examples/echo.bp", "--input", "first-char-only"],
        );
        Command::new(exe_name)
            .write_stdin(
                "pomme is cute\n"
                    .chars()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("pomme is cute\n");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn input_byte_as_number() {
        let exe_name = "./input_byte_as_number.out";
        compile(
            exe_name,
            &["./examples/echo.bp", "--input", "byte-as-number"],
        );
        Command::new(exe_name)
            .write_stdin(
                "pomme is cute\n"
                    .chars()
                    .map(|x| format!("{}\n", x as u8))
                    .collect::<String>(),
            )
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("pomme is cute\n");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn input_length_overflow() {
        let exe_name = "./input_length_overflow.out";
        compile(
            exe_name,
            &["./examples/echo.bp", "--input", "first-char-only"],
        );
        Command::new(exe_name)
            .write_stdin("ppppppppp\no\n\0\n\n")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("po\0\n");
        fs::remove_file(exe_name).unwrap();
    }

    // :3c
    #[test]
    fn output() {
        let exe_name = "./output.out";
        compile(exe_name, &["./examples/tests/output.bp"]);
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("\0");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn output_normal_mode() {
        let exe_name = "./output_normal_mode.out";
        compile(exe_name, &["./examples/echo.bp", "--output", "normal"]);
        Command::new(exe_name)
            .write_stdin("pomme is cute\n")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("pomme is cute\n");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn output_byte_as_number() {
        let exe_name = "./output_byte_as_number.out";
        compile(
            exe_name,
            &["./examples/echo.bp", "--output", "byte-as-number"],
        );
        Command::new(exe_name)
            .write_stdin("C\n")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("67\n10\n");
        fs::remove_file(exe_name).unwrap();
    }

    // nya :3
    #[test]
    fn useless_loop() {
        let exe_name = "./useless_loop.out";
        compile(exe_name, &["./examples/tests/useless_loop.bp"]);
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("\0");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn useful_loop() {
        let exe_name = "./useful_loop.out";
        compile(exe_name, &["./examples/tests/useful_loop.bp"]);
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("\n");
        fs::remove_file(exe_name).unwrap();
    }

    // max array size
    #[test]
    fn max_array_size_border() {
        let exe_name = "./max_array_size_border.out";
        compile(
            exe_name,
            &[
                "./examples/tests/max_array_size_border.bp",
                "--max-array-size",
                "10",
            ],
        );
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn max_array_size_overflow() {
        let exe_name = "./max_array_size_overflow.out";
        compile(
            exe_name,
            &[
                "./examples/tests/max_array_size_border.bp",
                "--max-array-size",
                "9",
            ],
        );
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .failure()
            .code(1)
            .stdout("");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn array_empty() {
        let exe_name = "./array_empty.out";
        compile(
            exe_name,
            &[
                "./examples/tests/array_empty.bp",
                "--max-array-size",
                "1000000",
            ],
        );
        Command::new(exe_name)
            .write_stdin("")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .failure()
            .code(1)
            .stdout("");
        fs::remove_file(exe_name).unwrap();
    }

    // newline zero
    #[test]
    fn newline_zero_normal_mode() {
        // normal mode for both input and output
        let exe_name = "./newline_zero_normal_mode.out";
        compile(
            exe_name,
            &[
                "./examples/tests/newline_zero.bp",
                "--newline-zero",
                "--input",
                "normal",
                "--output",
                "normal",
            ],
        );
        Command::new(exe_name)
            .write_stdin("\n\0")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("\n\0\n\0");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn newline_zero_byte_as_number() {
        // byte-as-number for both input and output
        let exe_name = "./newline_zero_byte_as_number.out";
        compile(
            exe_name,
            &[
                "./examples/tests/newline_zero.bp",
                "--newline-zero",
                "--input",
                "byte-as-number",
                "--output",
                "byte-as-number",
            ],
        );
        Command::new(exe_name)
            .write_stdin("10\n0\n")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("10\n0\n10\n0\n");
        fs::remove_file(exe_name).unwrap();
    }
    #[test]
    fn newline_zero_first_char_only() {
        // first-char-only mode for input and normal mode for output
        let exe_name = "./newline_zero_first_char_only.out";
        compile(
            exe_name,
            &[
                "./examples/tests/newline_zero.bp",
                "--newline-zero",
                "--input",
                "first-char-only",
            ],
        );
        Command::new(exe_name)
            .write_stdin("\n\0\n")
            .timeout(std::time::Duration::from_secs(1))
            .assert()
            .success()
            .code(0)
            .stdout("\n\0\n\0");
        fs::remove_file(exe_name).unwrap();
    }
}
