mod interpreter;
mod parser;
use std::{path::PathBuf};

use crate::{interpreter::*, parser::parse_file};
use clap::{Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: PathBuf,
}

fn main() {
    let args = Args::parse();
    let file_path = &args.path;

    let instructions = parse_file(file_path);
    let array = interpreter(instructions);
    println!("\nprogram ended\narray: {:?}", array);
}
