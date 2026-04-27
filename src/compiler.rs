use std::{ops::Add};

use crate::parser::Instruction;

pub fn compiler(instructions: Vec<Instruction>, max_array_size: Option<u32>) -> String {
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
