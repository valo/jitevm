use revm::interpreter::{
    instructions::{
        arithmetic, bitwise,
        control::{self, jumpi_without_pc},
        memory, stack,
    },
    Host, Interpreter,
};

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

pub fn fib(interpreter: &mut Interpreter, host: &mut dyn Host) {
    let mut jump: usize = 0;

    loop {
        match jump {
            0 => {
                stack::push_slice::<2>(interpreter, host, &[0x4E, 0x20]); // 20000
                stack::push_slice::<1>(interpreter, host, &[0]);
                stack::push_slice::<1>(interpreter, host, &[1]);
                jump = 7;
            }
            7 => {
                control::jumpdest(interpreter, host);
                stack::dup::<3>(interpreter, host);
                bitwise::iszero(interpreter, host);
                stack::push_slice::<1>(interpreter, host, &[1]);
                if control::jumpi_without_pc(interpreter, host).unwrap() {
                    jump = 28;
                } else {
                    jump = 13;
                }
            }
            13 => {
                stack::dup::<2>(interpreter, host);
                stack::dup::<2>(interpreter, host);
                arithmetic::wrapped_add(interpreter, host);
                stack::swap::<2>(interpreter, host);
                stack::pop(interpreter, host);
                stack::swap::<1>(interpreter, host);
                stack::swap::<2>(interpreter, host);
                stack::push_slice::<1>(interpreter, host, &[1]);
                stack::swap::<1>(interpreter, host);
                arithmetic::wrapping_sub(interpreter, host);
                stack::swap::<2>(interpreter, host);
                stack::push_slice::<1>(interpreter, host, &[7]);
                control::jump_without_pc(interpreter, host);
                jump = 7;
            }
            28 => {
                control::jumpdest(interpreter, host);
                stack::swap::<2>(interpreter, host);
                stack::pop(interpreter, host);
                stack::pop(interpreter, host);
                stack::push_slice::<1>(interpreter, host, &[0]);
                memory::mstore(interpreter, host);
                stack::push_slice::<1>(interpreter, host, &[32]);
                stack::push_slice::<1>(interpreter, host, &[0]);
                control::ret(interpreter, host);
                break;
            }
            _ => panic!("Invalid jump destination"),
        }
    }
}

pub fn fib_repeated(interpreter: &mut Interpreter, host: &mut dyn Host) {
    let mut jump = 0;

    loop {
        match jump {
            0 => {
                stack::push_slice::<2>(interpreter, host, &[0x27, 0x10]);
                jump = 3;
            }
            3 => {
                control::jumpdest(interpreter, host);
                stack::dup::<1>(interpreter, host);
                bitwise::iszero(interpreter, host);
                stack::push_slice::<1>(interpreter, host, &[48]);
                if jumpi_without_pc(interpreter, host).unwrap() {
                    jump = 48;
                } else {
                    jump = 13;
                }
            }
            13 => {
                stack::push_slice::<1>(interpreter, host, &[1]);
                stack::swap::<1>(interpreter, host);
                arithmetic::wrapping_sub(interpreter, host);
                stack::push_slice::<1>(interpreter, host, &[53]);
                stack::push_slice::<1>(interpreter, host, &[0]);
                stack::push_slice::<1>(interpreter, host, &[1]);
                jump = 19;
            }
            19 => {
                control::jumpdest(interpreter, host);
                stack::dup::<3>(interpreter, host);
                bitwise::iszero(interpreter, host);
                stack::push_slice::<1>(interpreter, host, &[40]);
                if jumpi_without_pc(interpreter, host).unwrap() {
                    jump = 40;
                } else {
                    jump = 25;
                }
            }
            25 => {
                stack::dup::<2>(interpreter, host);
                stack::dup::<2>(interpreter, host);
                arithmetic::wrapped_add(interpreter, host);
                stack::swap::<2>(interpreter, host);
                stack::pop(interpreter, host);
                stack::swap::<1>(interpreter, host);
                stack::swap::<2>(interpreter, host);
                stack::push_slice::<1>(interpreter, host, &[1]);
                stack::swap::<1>(interpreter, host);
                arithmetic::wrapping_sub(interpreter, host);
                stack::swap::<2>(interpreter, host);
                stack::push_slice::<1>(interpreter, host, &[19]);
                control::jump_without_pc(interpreter, host);
                jump = 19;
            }
            40 => {
                control::jumpdest(interpreter, host);
                stack::swap::<2>(interpreter, host);
                stack::pop(interpreter, host);
                stack::pop(interpreter, host);
                stack::pop(interpreter, host);
                stack::push_slice::<1>(interpreter, host, &[3]);
                control::jump_without_pc(interpreter, host);
                jump = 3;
            }
            48 => {
                control::jumpdest(interpreter, host);
                stack::push_slice::<1>(interpreter, host, &[0]);
                memory::mstore(interpreter, host);
                stack::push_slice::<1>(interpreter, host, &[32]);
                stack::push_slice::<1>(interpreter, host, &[0]);
                control::ret(interpreter, host);
                break;
            }
            _ => {
                panic!("Invalid jump destination")
            }
        }
    }
}
