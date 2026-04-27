mod compiler;
mod interpreter;
mod parser;
mod bf_parser;
use std::path::PathBuf;

use crate::{interpreter::*};
use clap::{Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// the path to the brainpurr file
    path: PathBuf,

    /// shows the array at the end of the program
    #[arg(long)]
    show_final_array: bool,

    /// to interpret the file as brainfuck instead of brainpurr
    #[arg(long)]
    from_brainfuck: bool,

    /// outputs the brainpurr code instead of running it (useful to translate brainfuck into brainpurr)
    #[arg(long)]
    to_brainpurr: bool,

    /// outputs the code as brainfuck instead of running it (useful to translate brainpurr into brainfuck)
    #[arg(long)]
    to_brainfuck: bool,

    /// outputs the code as c instead of running it (useful to translate brainpurr into c)
    #[arg(long)]
    to_c: bool,

    /// compile the code (requires gcc)
    #[arg(long)]
    compile: bool,
}

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
        return println!("{}", compiler::compile_to_c(&instructions, None));
    }
    if args.compile {
        return compiler::compile_to_file(&instructions, None);
    }

    let array = interpreter(instructions);
    if args.show_final_array {
        println!("\nfinal array: {:?}", array);
    }
}
