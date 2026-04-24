mod interpreter;
mod parser;
use crate::{interpreter::*, parser::parse_file};

fn main() {
    let instructions = parse_file("test.bf");
    interpreter(instructions);
}
