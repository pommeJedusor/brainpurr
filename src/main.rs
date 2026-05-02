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
    if args.to_brainpurr {
        return println!("{}", parser::unparse(instructions));
    }
    if args.to_brainfuck {
        return println!("{}", bf_parser::unparse(instructions));
    }
    if args.to_c {
        return println!("{}", compiler::compile_to_c(&instructions, &args));
    }
    if args.compile {
        return compiler::compile_to_file(&instructions, &args);
    }

    let array = interpreter(instructions, &args, &mut std::io::stdout());
    if args.show_final_array {
        println!("\nfinal array: {:?}", array);
    }
}
