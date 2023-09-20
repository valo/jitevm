use revm::interpreter::{
    instructions::{arithmetic, bitwise, control, memory, stack},
    Host, Interpreter,
};

#[inline(always)]
fn step(interpreter: &mut Interpreter) {
    interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.offset(1) };
}

fn jumpi(
    interpreter: &mut Interpreter,
    host: &mut dyn Host,
    branch_a: usize,
    branch_b: usize,
) -> usize {
    let ip_before_jump = interpreter.instruction_pointer;
    control::jumpi(interpreter, host);

    let jump_destination =
        unsafe { interpreter.instruction_pointer.offset_from(ip_before_jump) as isize };

    if jump_destination != 0 {
        branch_a
    } else {
        branch_b
    }
}

pub fn fib(interpreter: &mut Interpreter, host: &mut dyn Host) {
    let mut jump: usize = 0;

    loop {
        match jump {
            0 => {
                step(interpreter);
                stack::push::<2>(interpreter, host);
                step(interpreter);
                stack::push::<1>(interpreter, host);
                step(interpreter);
                stack::push::<1>(interpreter, host);
                jump = 7;
            }
            7 => {
                step(interpreter);
                control::jumpdest(interpreter, host);
                step(interpreter);
                stack::dup::<3>(interpreter, host);
                step(interpreter);
                bitwise::iszero(interpreter, host);
                step(interpreter);
                stack::push::<1>(interpreter, host);
                step(interpreter);
                jump = jumpi(interpreter, host, 28, 13);
            }
            13 => {
                step(interpreter);
                stack::dup::<2>(interpreter, host);
                step(interpreter);
                stack::dup::<2>(interpreter, host);
                step(interpreter);
                arithmetic::wrapped_add(interpreter, host);
                step(interpreter);
                stack::swap::<2>(interpreter, host);
                step(interpreter);
                stack::pop(interpreter, host);
                step(interpreter);
                stack::swap::<1>(interpreter, host);
                step(interpreter);
                stack::swap::<2>(interpreter, host);
                step(interpreter);
                stack::push::<1>(interpreter, host);
                step(interpreter);
                stack::swap::<1>(interpreter, host);
                step(interpreter);
                arithmetic::wrapping_sub(interpreter, host);
                step(interpreter);
                stack::swap::<2>(interpreter, host);
                step(interpreter);
                stack::push::<1>(interpreter, host);
                step(interpreter);
                control::jump(interpreter, host);
                jump = 7;
            }
            28 => {
                step(interpreter);
                control::jumpdest(interpreter, host);
                step(interpreter);
                stack::swap::<2>(interpreter, host);
                step(interpreter);
                stack::pop(interpreter, host);
                step(interpreter);
                stack::pop(interpreter, host);
                step(interpreter);
                stack::push::<1>(interpreter, host);
                step(interpreter);
                memory::mstore(interpreter, host);
                step(interpreter);
                stack::push::<1>(interpreter, host);
                step(interpreter);
                stack::push::<1>(interpreter, host);
                step(interpreter);
                control::ret(interpreter, host);
                break;
            }
            _ => panic!("Invalid jump destination"),
        }
    }
}
