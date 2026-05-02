use std::path::PathBuf;

use clap::{Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// the path to the brainpurr file
    pub path: PathBuf,

    /// shows the array at the end of the program
    #[arg(long)]
    pub show_final_array: bool,

    /// interprets the file as brainfuck instead of brainpurr
    #[arg(long)]
    pub from_brainfuck: bool,

    /// outputs the brainpurr code instead of running it (useful to translate brainfuck into brainpurr)
    #[arg(long)]
    pub to_brainpurr: bool,

    /// outputs the code as brainfuck instead of running it (useful to translate brainpurr into brainfuck)
    #[arg(long)]
    pub to_brainfuck: bool,

    /// outputs the code as c instead of running it (useful to translate brainpurr into c)
    #[arg(long)]
    pub to_c: bool,

    /// compiles the code (requires gcc)
    #[arg(long)]
    pub compile: bool,

    /// the length of the array (only works for compiler)
    #[arg(long, default_value_t=67000)]
    pub max_array_size: u32,

    /// arguments to pass to gcc when compiling the code from c to binary
    #[arg(long, allow_hyphen_values = true)]
    pub gcc_args: Option<String>,

    /// input method
    #[clap(value_enum)]
    #[arg(long, default_value_t=InputMethod::Normal)]
    pub input: InputMethod,

    /// output method
    #[clap(value_enum)]
    #[arg(long, default_value_t=OutputMethod::Normal)]
    pub output: OutputMethod,

    /// replaces '\n' character ascii value (10) by 0 for input and output
    #[arg(long)]
    pub newline_zero: bool,
}

#[derive(clap::ValueEnum, Debug, Clone, Hash)]
pub enum InputMethod {
    /// interprets the input byte per byte including the \n
    Normal,
    /// only takes the first byte from each line
    FirstCharOnly,
    /// interprets the line as a number represented number (must be between 0 and 255 included)
    ByteAsNumber,
}

#[derive(clap::ValueEnum, Debug, Clone, Hash)]
pub enum OutputMethod {
    /// outputs each byte as ascii
    Normal,
    /// outputs each byte as a number
    ByteAsNumber,
}

pub trait InterpreterArgs{
    fn get_input_method(&self) -> &InputMethod;
    fn get_output_method(&self) -> &OutputMethod;
    fn get_newline_zero(&self) -> bool;
}

impl InterpreterArgs for Args {
    fn get_input_method(&self) -> &InputMethod { &self.input }
    fn get_output_method(&self) -> &OutputMethod { &self.output }
    fn get_newline_zero(&self) -> bool { self.newline_zero }
}
