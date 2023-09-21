use revm::interpreter::{
    instructions::{
        arithmetic, bitwise,
        control::{self, jumpi_without_pc},
        memory, stack,
    },
    Host, InstructionResult, Interpreter,
};

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

pub fn fib<const CHECKED: bool>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    let mut jump: usize = 0;

    loop {
        match jump {
            0 => {
                stack::push_slice::<2>(interpreter, host, &[0x4E, 0x20]); // 20000
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[0]);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[1]);
                check_instruction_result!(interpreter);
                jump = 7;
            }
            7 => {
                control::jumpdest(interpreter, host);
                check_instruction_result!(interpreter);
                stack::dup::<3>(interpreter, host);
                check_instruction_result!(interpreter);
                bitwise::iszero(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[28]);
                check_instruction_result!(interpreter);
                let jump_result = control::jumpi_without_pc(interpreter, host);
                check_instruction_result!(interpreter);
                jump = jump_result.unwrap();

                if jump == 0 {
                    jump = 13;
                }
            }
            13 => {
                stack::dup::<2>(interpreter, host);
                check_instruction_result!(interpreter);
                stack::dup::<2>(interpreter, host);
                check_instruction_result!(interpreter);
                arithmetic::wrapped_add(interpreter, host);
                check_instruction_result!(interpreter);
                stack::swap::<2>(interpreter, host);
                check_instruction_result!(interpreter);
                stack::pop(interpreter, host);
                check_instruction_result!(interpreter);
                stack::swap::<1>(interpreter, host);
                check_instruction_result!(interpreter);
                stack::swap::<2>(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[1]);
                check_instruction_result!(interpreter);
                stack::swap::<1>(interpreter, host);
                check_instruction_result!(interpreter);
                arithmetic::wrapping_sub(interpreter, host);
                check_instruction_result!(interpreter);
                stack::swap::<2>(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[7]);
                check_instruction_result!(interpreter);
                let jump_result = control::jump_without_pc(interpreter, host);
                check_instruction_result!(interpreter);

                jump = jump_result.unwrap();
            }
            28 => {
                control::jumpdest(interpreter, host);
                check_instruction_result!(interpreter);
                stack::swap::<2>(interpreter, host);
                check_instruction_result!(interpreter);
                stack::pop(interpreter, host);
                check_instruction_result!(interpreter);
                stack::pop(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[0]);
                check_instruction_result!(interpreter);
                memory::mstore(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[32]);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[0]);
                check_instruction_result!(interpreter);
                control::ret(interpreter, host);
                check_instruction_result!(interpreter);
                break;
            }
            _ => panic!("Invalid jump destination"),
        }
    }
}

pub fn fib_repeated<const CHECKED: bool>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    let mut jump = 0;

    loop {
        match jump {
            0 => {
                stack::push_slice::<2>(interpreter, host, &[0x27, 0x10]);
                check_instruction_result!(interpreter);
                jump = 3;
            }
            3 => {
                control::jumpdest(interpreter, host);
                check_instruction_result!(interpreter);
                stack::dup::<1>(interpreter, host);
                check_instruction_result!(interpreter);
                bitwise::iszero(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[48]);
                check_instruction_result!(interpreter);
                let jump_result = control::jumpi_without_pc(interpreter, host);
                check_instruction_result!(interpreter);
                jump = jump_result.unwrap();

                if jump == 0 {
                    jump = 13;
                }
            }
            13 => {
                stack::push_slice::<1>(interpreter, host, &[1]);
                check_instruction_result!(interpreter);
                stack::swap::<1>(interpreter, host);
                check_instruction_result!(interpreter);
                arithmetic::wrapping_sub(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[53]);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[0]);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[1]);
                check_instruction_result!(interpreter);
                jump = 19;
            }
            19 => {
                control::jumpdest(interpreter, host);
                check_instruction_result!(interpreter);
                stack::dup::<3>(interpreter, host);
                check_instruction_result!(interpreter);
                bitwise::iszero(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[40]);
                check_instruction_result!(interpreter);
                let jump_result = control::jumpi_without_pc(interpreter, host);
                check_instruction_result!(interpreter);
                jump = jump_result.unwrap();

                if jump == 0 {
                    jump = 25;
                }
            }
            25 => {
                stack::dup::<2>(interpreter, host);
                check_instruction_result!(interpreter);
                stack::dup::<2>(interpreter, host);
                check_instruction_result!(interpreter);
                arithmetic::wrapped_add(interpreter, host);
                check_instruction_result!(interpreter);
                stack::swap::<2>(interpreter, host);
                check_instruction_result!(interpreter);
                stack::pop(interpreter, host);
                check_instruction_result!(interpreter);
                stack::swap::<1>(interpreter, host);
                check_instruction_result!(interpreter);
                stack::swap::<2>(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[1]);
                check_instruction_result!(interpreter);
                stack::swap::<1>(interpreter, host);
                check_instruction_result!(interpreter);
                arithmetic::wrapping_sub(interpreter, host);
                check_instruction_result!(interpreter);
                stack::swap::<2>(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[19]);
                check_instruction_result!(interpreter);
                let jump_result = control::jump_without_pc(interpreter, host);
                check_instruction_result!(interpreter);

                jump = jump_result.unwrap();
            }
            40 => {
                control::jumpdest(interpreter, host);
                check_instruction_result!(interpreter);
                stack::swap::<2>(interpreter, host);
                check_instruction_result!(interpreter);
                stack::pop(interpreter, host);
                check_instruction_result!(interpreter);
                stack::pop(interpreter, host);
                check_instruction_result!(interpreter);
                stack::pop(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[3]);
                check_instruction_result!(interpreter);
                let jump_result = control::jump_without_pc(interpreter, host);
                check_instruction_result!(interpreter);

                jump = jump_result.unwrap();
            }
            48 => {
                control::jumpdest(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[0]);
                check_instruction_result!(interpreter);
                memory::mstore(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[32]);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[0]);
                check_instruction_result!(interpreter);
                control::ret(interpreter, host);
                return;
            }
            _ => {
                panic!("Invalid jump destination")
            }
        }
    }
}
