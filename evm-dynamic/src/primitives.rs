use revm::interpreter::{
    instructions::{
        arithmetic, bitwise,
        control::{self},
        memory, stack,
    },
    Host, InstructionResult, Interpreter,
};

pub trait CompiledEVM<const CHECKED: bool, SPEC: revm::primitives::Spec> {
    fn call(&self, interpreter: &mut Interpreter, host: &mut dyn Host);
}

#[macro_export]
macro_rules! check_instruction_result {
    ($interpreter:expr) => {
        if CHECKED && $interpreter.instruction_result != InstructionResult::Continue {
            return;
        }
    };
}

#[inline(always)]
fn print_stack(interpreter: &mut Interpreter) {
    println!(
        "Stack: {:?}",
        interpreter
            .stack
            .data()
            .iter()
            .map(|x| x.as_limbs()[0])
            .collect::<Vec<u64>>()
    );
}
