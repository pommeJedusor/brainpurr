mod args;
mod compiler;
mod interpreter;
mod parser;
mod bf_parser;

use crate::{interpreter::*, args::*};
use clap::{Parser};


fn main() {
    let args = Args::parse();

    let path = &args.path;
    let instructions = match &args.from_brainfuck {
        false => parser::parse_file(path),
        true => bf_parser::parse_file(path),
    };
    let input_method = &args.input;
    let output_method = &args.output;
    let gcc_args = args.gcc_args.clone().unwrap_or("".to_string());
    // TODO implement max_array_size for interpreter
    let max_array_size = args.max_array_size;

    if args.to_brainpurr {
        return println!("{}", parser::unparse(instructions));
    }
    if args.to_brainfuck {
        return println!("{}", bf_parser::unparse(instructions));
    }
    if args.to_c {
        return println!("{}", compiler::compile_to_c(&instructions, Some(max_array_size), &input_method, &output_method));
    }
    if args.compile {
        let gcc_args = gcc_args.split(" ").filter(|x| x != &"").collect::<Vec<&str>>();
        return compiler::compile_to_file(&instructions, Some(max_array_size), &input_method, &output_method, &gcc_args);
    }

    let array = interpreter(instructions, &args, &mut std::io::stdout());
    if args.show_final_array {
        println!("\nfinal array: {:?}", array);
    }
}
