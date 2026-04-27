use std::{fs::{self, File}, ops::Add, io::Write, process::Command};

use crate::parser::Instruction;

pub fn compile_to_c(instructions: &Vec<Instruction>, max_array_size: Option<u32>) -> String {
    let max_array_size = max_array_size.unwrap_or(67000);

    let c_file = "#include <stdio.h>\n".to_string();
    let c_file = c_file.add(&format!("const unsigned long ARRAY_LENGTH = {};\n", max_array_size));
    let c_file = c_file.add("void main(){\nchar array[ARRAY_LENGTH];\nfor (int i=0;i<ARRAY_LENGTH;i++){\narray[i] = 0;\n}\nunsigned int pointer = 0;\n");

    let c_file = c_file.add(&instructions.iter().map(|x| match x{
        Instruction::PointerIncrement => "pointer++;",
        Instruction::PointerDecrement => "pointer--;",
        Instruction::ByteIncrement => "array[pointer]++;",
        Instruction::ByteDecrement => "array[pointer]--;",
        Instruction::ByteInput => "scanf(\"%c\", &array[pointer]);",
        Instruction::ByteOutput => "printf(\"%c\", array[pointer]);",
        Instruction::OpenLoop(_) => "while (array[pointer] != 0){",
        Instruction::CloseLoop(_) => "}",
    }).collect::<Vec<&str>>().join("\n"));

    let c_file = c_file.add("}");

    c_file
}

pub fn compile_to_file(instructions: &Vec<Instruction>, max_array_size: Option<u32>){
    let c_code = compile_to_c(instructions, max_array_size);
    let c_file_name = "temp-jq7uvwn9up6u1wqpg756wh3flkyrmb9qwogro9j9.c";
    let mut file = File::create(c_file_name).expect("failed to create temporary file for compiling");
    let _ = write!(file, "{}", c_code);
    let _ = Command::new("gcc")
        .args([c_file_name])
        .output()
        .expect("failed to compile the code using gcc (gcc is required to run this command)");
    fs::remove_file(c_file_name).expect("failed to delete temporary file for compiling");
    return;
}
