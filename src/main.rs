#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![cfg_attr(not(test), deny(unused_must_use))]

mod args;
mod bf_parser;
mod compiler;
mod error;
mod interpreter;
mod parser;

use crate::{args::Args, interpreter::interpreter};
use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let path = &args.path;
    let instructions = match &args.from_brainfuck {
        false => parser::parse_file(path)?,
        true => bf_parser::parse_file(path)?,
    };
    if args.to_brainpurr {
        println!("{}", parser::unparse(instructions));
        return Ok(());
    }
    if args.to_brainfuck {
        println!("{}", bf_parser::unparse(instructions));
        return Ok(());
    }
    if args.to_c {
        println!("{}", compiler::compile_to_c(&instructions, &args));
        return Ok(());
    }
    if args.compile {
        compiler::compile_to_file(&instructions, &args)?;
        return Ok(());
    }

    let array = interpreter(instructions, &args, &mut std::io::stdout())?;
    if args.show_final_array {
        println!("\nfinal array: {:?}", array);
    }
    Ok(())
}
