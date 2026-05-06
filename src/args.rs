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

    /// the length of the array, by default unlimited for the interpreter and 67_000 for the compiler
    #[arg(long)]
    pub max_array_size: Option<u32>,

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

    /// replaces '\n' character ascii value (10) by 0 for input and output and the \0 character
    /// value by 10
    #[arg(long)]
    pub newline_zero: bool,

    #[clap(value_enum)]
    #[arg(long, default_value_t=PointerWrapMode::Crash)]
    pub pointer_wrap: PointerWrapMode,
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

#[derive(clap::ValueEnum, Debug, Clone, Hash)]
pub enum PointerWrapMode {
    /// crashes if the pointer goes under 0 or higher than the array limit size
    Crash,
    /// doesn't check for overflow/undeflow, the behavior is undefined in case it happens, might
    /// make your code run faster because doesn't check for it
    Unsafe,
    /// in case of overflow goes back to 0 (e.g.array size limit = 10, pointer = 9 if it gets
    /// increase by 1 it will become 0) and vice-versa
    WrapAround,
    /// in case of overflow/underflow doesn't change the value (e.g. if pointer = 0 and it gets
    /// decrease it will stay at 0)
    Stick,
}

pub trait InterpreterArgs{
    fn get_input_method(&self) -> &InputMethod;
    fn get_output_method(&self) -> &OutputMethod;
    fn get_newline_zero(&self) -> bool;
    fn get_max_array_size(&self) -> Option<u32>;
    fn get_pointer_wrap_mode(&self) -> &PointerWrapMode;
}

impl InterpreterArgs for Args {
    fn get_input_method(&self) -> &InputMethod { &self.input }
    fn get_output_method(&self) -> &OutputMethod { &self.output }
    fn get_newline_zero(&self) -> bool { self.newline_zero }
    fn get_max_array_size(&self) -> Option<u32> { self.max_array_size }
    fn get_pointer_wrap_mode(&self) -> &PointerWrapMode { &self.pointer_wrap }
}

pub trait CompilerArgs{
    fn get_input_method(&self) -> &InputMethod;
    fn get_output_method(&self) -> &OutputMethod;
    fn get_newline_zero(&self) -> bool;
    fn get_max_array_size(&self) -> u32;
    fn get_gcc_args(&self) -> Option<String>;
    fn get_pointer_wrap_mode(&self) -> &PointerWrapMode;
}

impl CompilerArgs for Args {
    fn get_input_method(&self) -> &InputMethod { &self.input }
    fn get_output_method(&self) -> &OutputMethod { &self.output }
    fn get_newline_zero(&self) -> bool { self.newline_zero }
    fn get_max_array_size(&self) -> u32 { self.max_array_size.unwrap_or(67000) }
    fn get_gcc_args(&self) -> Option<String> { self.gcc_args.clone() }
    fn get_pointer_wrap_mode(&self) -> &PointerWrapMode { &self.pointer_wrap }
}
