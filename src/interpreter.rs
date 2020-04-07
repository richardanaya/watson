use crate::core::*;

pub struct Interpreter(Program);

impl Interpreter {
    pub fn from_program(p: Program) -> Self {
        Interpreter(p)
    }
}
