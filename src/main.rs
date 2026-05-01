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

    /// interprets the file as brainfuck instead of brainpurr
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

    /// compiles the code (requires gcc)
    #[arg(long)]
    compile: bool,

    /// arguments to pass to gcc when compiling the code from c to binary
    #[arg(long, allow_hyphen_values = true)]
    gcc_args: Option<String>,

    /// input method
    #[clap(value_enum)]
    #[arg(long, default_value_t=InputMethod::Normal)]
    input: InputMethod,

    /// output method
    #[clap(value_enum)]
    #[arg(long, default_value_t=OutputMethod::Normal)]
    output: OutputMethod,
}

#[derive(clap::ValueEnum, Debug, Clone, Hash)]
enum InputMethod {
    /// interprets the input byte per byte including the \n
    Normal,
    /// only takes the first byte from each line
    FirstCharOnly,
    /// interprets the line as a number represented number (must be between 0 and 255 included)
    ByteAsNumber,
}

#[derive(clap::ValueEnum, Debug, Clone, Hash)]
enum OutputMethod {
    /// outputs each byte as ascii
    Normal,
    /// outputs each byte as a number
    ByteAsNumber,
}

fn main() {
    let args = Args::parse();

    let path = &args.path;
    let instructions = match &args.from_brainfuck {
        false => parser::parse_file(path),
        true => bf_parser::parse_file(path),
    };
    let input_method = args.input;
    let output_method = args.output;
    let gcc_args = args.gcc_args.unwrap_or("".to_string());

    if args.to_brainpurr {
        return println!("{}", parser::unparse(instructions));
    }
    if args.to_brainfuck {
        return println!("{}", bf_parser::unparse(instructions));
    }
    if args.to_c {
        return println!("{}", compiler::compile_to_c(&instructions, None, &input_method, &output_method));
    }
    if args.compile {
        let gcc_args = gcc_args.split(" ").filter(|x| x != &"").collect::<Vec<&str>>();
        return compiler::compile_to_file(&instructions, None, &input_method, &output_method, &gcc_args);
    }

    let array = interpreter(instructions, &input_method, &output_method, &mut std::io::stdout());
    if args.show_final_array {
        println!("\nfinal array: {:?}", array);
    }
}
