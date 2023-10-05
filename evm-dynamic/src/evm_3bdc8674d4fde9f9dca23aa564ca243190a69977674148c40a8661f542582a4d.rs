use revm::interpreter::{
    instructions::{
        arithmetic, bitwise, control,
        memory, stack, system, host_env,
        host
    },
    primitives::Spec,
    Host, InstructionResult, Interpreter,
};
    
use crate::check_instruction_result;
pub fn call<const CHECKED: bool, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    let mut jump: usize = 0;
    
    loop {
        match jump {
            0 => {
                stack::push_slice::<2>(interpreter, host, &[0x4e, 0x20]);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[0x00]);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[0x01]);
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
                stack::push_slice::<1>(interpreter, host, &[0x1c]);
                check_instruction_result!(interpreter);
                let jump_result = control::jumpi_without_pc(interpreter, host);
                check_instruction_result!(interpreter);
                jump = jump_result.unwrap();
                if jump == 0 {
                    jump = 12;
                }
            }
            12 => {
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
                stack::push_slice::<1>(interpreter, host, &[0x01]);
                check_instruction_result!(interpreter);
                stack::swap::<1>(interpreter, host);
                check_instruction_result!(interpreter);
                arithmetic::wrapping_sub(interpreter, host);
                check_instruction_result!(interpreter);
                stack::swap::<2>(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[0x07]);
                check_instruction_result!(interpreter);
                let jump_result = control::jump_without_pc(interpreter, host);
                check_instruction_result!(interpreter);
                jump = jump_result.unwrap();
            }
            27 => {
                jump = 28;
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
                stack::push_slice::<1>(interpreter, host, &[0x00]);
                check_instruction_result!(interpreter);
                memory::mstore(interpreter, host);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[0x20]);
                check_instruction_result!(interpreter);
                stack::push_slice::<1>(interpreter, host, &[0x00]);
                check_instruction_result!(interpreter);
                control::ret(interpreter, host);
                check_instruction_result!(interpreter);
                return;
            }
            _ => panic!("Invalid jump destination"),
        }
    }
}