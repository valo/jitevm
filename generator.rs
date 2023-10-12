use std::{
    fs::File,
    io::{Error, Write},
};

fn format_slice(slice: &[u8]) -> String {
    let result = slice
        .iter()
        .map(|x| format!("0x{:02x}", x))
        .collect::<Vec<String>>()
        .join(", ");
    format!("[{result}]")
}

fn output_push(
    file: &mut File,
    code: &[u8],
    instruction_index: &mut usize,
    size: usize,
) -> Result<(), Error> {
    *instruction_index += 1;
    file.write_all(
        format!(
            "                stack::push_slice::<{}>(interpreter, host, &{});\n",
            size,
            format_slice(&code[*instruction_index..(*instruction_index + size)])
        )
        .as_bytes(),
    )?;
    file.write_all(b"                check_instruction_result!(interpreter);\n")?;
    *instruction_index += size - 1;
    Ok(())
}

pub fn convert_evm_to_rust(file: &mut File, code: &[u8]) -> Result<(), Error> {
    file.write_all(
        b"use revm::interpreter::{
    instructions::{
        arithmetic, bitwise, control,
        memory, stack, system, host_env,
        host
    },
    primitives::Spec,
    Host, InstructionResult, Interpreter,
};
use crate::{check_instruction_result, primitives::CompiledEVM};

    
pub struct EVMCode;

impl<const CHECKED: bool, SPEC: Spec> CompiledEVM<CHECKED, SPEC> for EVMCode {
    fn call(&self, interpreter: &mut Interpreter, host: &mut dyn Host) {
    let mut jump: usize = 0;
    
    loop {
        match jump {
            0 => {\n",
    )?;

    let mut instruction_index: usize = 0;
    while instruction_index < code.len() {
        let current_op = code[instruction_index];

        match current_op {
            0x00 => {
                file.write_all(b"                control::stop(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
                file.write_all(b"                return;\n")?;
            }
            0x01 => {
                file.write_all(b"                arithmetic::wrapped_add(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x02 => {
                file.write_all(b"                arithmetic::wrapping_mul(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x03 => {
                file.write_all(b"                arithmetic::wrapping_sub(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x04 => {
                file.write_all(b"                arithmetic::div(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x05 => {
                file.write_all(b"                arithmetic::sdiv(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x06 => {
                file.write_all(b"                arithmetic::rem(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x07 => {
                file.write_all(b"                arithmetic::smod(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x08 => {
                file.write_all(b"                arithmetic::addmod(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x09 => {
                file.write_all(b"                arithmetic::mulmod(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x0A => {
                file.write_all(b"                arithmetic::exp::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x0B => {
                file.write_all(b"                arithmetic::signextend(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x10 => {
                file.write_all(b"                bitwise::lt(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x11 => {
                file.write_all(b"                bitwise::gt(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x12 => {
                file.write_all(b"                bitwise::slt(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x13 => {
                file.write_all(b"                bitwise::sgt(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x14 => {
                file.write_all(b"                bitwise::eq(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x15 => {
                file.write_all(b"                bitwise::iszero(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x16 => {
                file.write_all(b"                bitwise::bitand(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x17 => {
                file.write_all(b"                bitwise::bitor(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x18 => {
                file.write_all(b"                bitwise::bitxor(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x19 => {
                file.write_all(b"                bitwise::not(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x1A => {
                file.write_all(b"                bitwise::byte(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x1B => {
                file.write_all(b"                bitwise::shl::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x1C => {
                file.write_all(b"                bitwise::shr::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x1D => {
                file.write_all(b"                bitwise::sar::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x20 => {
                file.write_all(b"                system::keccak256(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x30 => {
                file.write_all(b"                system::address(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x31 => {
                file.write_all(b"                host::balance::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x32 => {
                file.write_all(b"                host_env::origin(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x33 => {
                file.write_all(b"                system::caller(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x34 => {
                file.write_all(b"                system::callvalue(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x35 => {
                file.write_all(b"                system::calldataload(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x36 => {
                file.write_all(b"                system::calldatasize(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x37 => {
                file.write_all(b"                system::calldatacopy(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x38 => {
                file.write_all(b"                system::codesize(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x39 => {
                file.write_all(b"                system::codecopy(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x3A => {
                file.write_all(b"                host_env::gasprice(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x3B => {
                file.write_all(b"                host::extcodesize::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x3C => {
                file.write_all(b"                host::extcodecopy::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x3D => {
                file.write_all(
                    b"                system::returndatasize::<SPEC>(interpreter, host);\n",
                )?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x3E => {
                file.write_all(
                    b"                system::returndatacopy::<SPEC>(interpreter, host);\n",
                )?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x3F => {
                file.write_all(b"                host::extcodehash::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x40 => {
                file.write_all(b"                host::blockhash(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x41 => {
                file.write_all(b"                host_env::coinbase(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x42 => {
                file.write_all(b"                host_env::timestamp(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x43 => {
                file.write_all(b"                host_env::number(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x44 => {
                file.write_all(
                    b"                host_env::difficulty::<SPEC>(interpreter, host);\n",
                )?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x45 => {
                file.write_all(b"                host_env::gaslimit(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x46 => {
                file.write_all(b"                host_env::chainid::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x47 => {
                file.write_all(b"                host::selfbalance::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x48 => {
                file.write_all(b"                host_env::basefee::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x49 => {
                file.write_all(
                    b"                host_env::blob_hash::<SPEC>(interpreter, host);\n",
                )?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x50 => {
                file.write_all(b"                stack::pop(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x51 => {
                file.write_all(b"                memory::mload(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x52 => {
                file.write_all(b"                memory::mstore(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x53 => {
                file.write_all(b"                memory::mstore8(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x54 => {
                file.write_all(b"                host::sload::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x55 => {
                file.write_all(b"                host::sstore::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x56 => {
                file.write_all(
                    b"                let jump_result = control::jump_without_pc(interpreter, host);\n",
                )?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
                file.write_all(b"                jump = jump_result.unwrap();\n")?;
                file.write_all(b"                continue;\n")?;
            }
            0x57 => {
                file.write_all(
                    b"                let jump_result = control::jumpi_without_pc(interpreter, host);\n",
                )?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
                file.write_all(b"                jump = jump_result.unwrap();\n")?;
                file.write_all(b"                if jump == 0 {\n")?;
                file.write_all(
                    format!("                    jump = {};\n", instruction_index).as_bytes(),
                )?;
                file.write_all(b"                }\n")?;
                file.write_all(b"            }\n")?;
                file.write_all(format!("            {} => {{\n", instruction_index).as_bytes())?;
            }
            0x58 => {
                file.write_all(b"                control::pc(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x59 => {
                file.write_all(b"                memory::msize(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x5A => {
                file.write_all(b"                system::gas(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x5B => {
                file.write_all(
                    format!("                jump = {};\n", instruction_index).as_bytes(),
                )?;
                file.write_all(b"            }\n")?;
                file.write_all(format!("            {} => {{\n", instruction_index).as_bytes())?;
                file.write_all(b"                control::jumpdest(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x5C => {
                file.write_all(b"                host::tload::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x5D => {
                file.write_all(b"                host::tstore::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x5E => {
                file.write_all(b"                memory::mcopy::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x5F => {
                file.write_all(b"                stack::push0::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x60 => {
                output_push(file, code, &mut instruction_index, 1)?;
            }
            0x61 => {
                output_push(file, code, &mut instruction_index, 2)?;
            }
            0x62 => {
                output_push(file, code, &mut instruction_index, 3)?;
            }
            0x63 => {
                output_push(file, code, &mut instruction_index, 4)?;
            }
            0x64 => {
                output_push(file, code, &mut instruction_index, 5)?;
            }
            0x65 => {
                output_push(file, code, &mut instruction_index, 6)?;
            }
            0x66 => {
                output_push(file, code, &mut instruction_index, 7)?;
            }
            0x67 => {
                output_push(file, code, &mut instruction_index, 8)?;
            }
            0x68 => {
                output_push(file, code, &mut instruction_index, 9)?;
            }
            0x69 => {
                output_push(file, code, &mut instruction_index, 10)?;
            }
            0x6A => {
                output_push(file, code, &mut instruction_index, 11)?;
            }
            0x6B => {
                output_push(file, code, &mut instruction_index, 12)?;
            }
            0x6C => {
                output_push(file, code, &mut instruction_index, 13)?;
            }
            0x6D => {
                output_push(file, code, &mut instruction_index, 14)?;
            }
            0x6E => {
                output_push(file, code, &mut instruction_index, 15)?;
            }
            0x6F => {
                output_push(file, code, &mut instruction_index, 16)?;
            }
            0x70 => {
                output_push(file, code, &mut instruction_index, 17)?;
            }
            0x71 => {
                output_push(file, code, &mut instruction_index, 18)?;
            }
            0x72 => {
                output_push(file, code, &mut instruction_index, 19)?;
            }
            0x73 => {
                output_push(file, code, &mut instruction_index, 20)?;
            }
            0x74 => {
                output_push(file, code, &mut instruction_index, 21)?;
            }
            0x75 => {
                output_push(file, code, &mut instruction_index, 22)?;
            }
            0x76 => {
                output_push(file, code, &mut instruction_index, 23)?;
            }
            0x77 => {
                output_push(file, code, &mut instruction_index, 24)?;
            }
            0x78 => {
                output_push(file, code, &mut instruction_index, 25)?;
            }
            0x79 => {
                output_push(file, code, &mut instruction_index, 26)?;
            }
            0x7A => {
                output_push(file, code, &mut instruction_index, 27)?;
            }
            0x7B => {
                output_push(file, code, &mut instruction_index, 28)?;
            }
            0x7C => {
                output_push(file, code, &mut instruction_index, 29)?;
            }
            0x7D => {
                output_push(file, code, &mut instruction_index, 30)?;
            }
            0x7E => {
                output_push(file, code, &mut instruction_index, 31)?;
            }
            0x7F => {
                output_push(file, code, &mut instruction_index, 32)?;
            }
            0x80 => {
                file.write_all(b"                stack::dup::<1>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x81 => {
                file.write_all(b"                stack::dup::<2>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x82 => {
                file.write_all(b"                stack::dup::<3>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x83 => {
                file.write_all(b"                stack::dup::<4>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x84 => {
                file.write_all(b"                stack::dup::<5>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x85 => {
                file.write_all(b"                stack::dup::<6>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x86 => {
                file.write_all(b"                stack::dup::<7>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x87 => {
                file.write_all(b"                stack::dup::<8>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x88 => {
                file.write_all(b"                stack::dup::<9>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x89 => {
                file.write_all(b"                stack::dup::<10>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x8A => {
                file.write_all(b"                stack::dup::<11>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x8B => {
                file.write_all(b"                stack::dup::<12>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x8C => {
                file.write_all(b"                stack::dup::<13>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x8D => {
                file.write_all(b"                stack::dup::<14>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x8E => {
                file.write_all(b"                stack::dup::<15>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x8F => {
                file.write_all(b"                stack::dup::<16>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x90 => {
                file.write_all(b"                stack::swap::<1>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x91 => {
                file.write_all(b"                stack::swap::<2>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x92 => {
                file.write_all(b"                stack::swap::<3>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x93 => {
                file.write_all(b"                stack::swap::<4>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x94 => {
                file.write_all(b"                stack::swap::<5>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x95 => {
                file.write_all(b"                stack::swap::<6>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x96 => {
                file.write_all(b"                stack::swap::<7>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x97 => {
                file.write_all(b"                stack::swap::<8>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x98 => {
                file.write_all(b"                stack::swap::<9>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x99 => {
                file.write_all(b"                stack::swap::<10>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x9A => {
                file.write_all(b"                stack::swap::<11>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x9B => {
                file.write_all(b"                stack::swap::<12>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x9C => {
                file.write_all(b"                stack::swap::<13>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x9D => {
                file.write_all(b"                stack::swap::<14>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x9E => {
                file.write_all(b"                stack::swap::<15>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0x9F => {
                file.write_all(b"                stack::swap::<16>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0xA0 => {
                file.write_all(b"                host::log::<0>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0xA1 => {
                file.write_all(b"                host::log::<1>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0xA2 => {
                file.write_all(b"                host::log::<2>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0xA3 => {
                file.write_all(b"                host::log::<3>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0xA4 => {
                file.write_all(b"                host::log::<4>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0xF0 => {
                file.write_all(b"                host::create(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0xF1 => {
                file.write_all(b"                host::call(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0xF2 => {
                file.write_all(b"                host::call_code(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0xF3 => {
                file.write_all(b"                control::ret(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
                file.write_all(b"                return;\n")?;
            }
            0xF4 => {
                file.write_all(b"                host::delegatecall(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0xF5 => {
                file.write_all(b"                host::create2(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0xFA => {
                file.write_all(b"                host::staticcall(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0xFD => {
                file.write_all(b"                control::revert::<SPEC>(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
                file.write_all(b"                return;\n")?;
            }
            0xFE => {
                file.write_all(b"                control::invalid(interpreter, host);\n")?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }
            0xFF => {
                file.write_all(
                    b"                host::selfdestruct::<SPEC>(interpreter, host);\n",
                )?;
                file.write_all(b"                check_instruction_result!(interpreter);\n")?;
            }

            _ => {
                println!(
                    "Unknown opcode at index {}: {}",
                    instruction_index, current_op
                )
            }
        }
        instruction_index += 1;
    }
    file.write_all(
        b"               }
                _ => panic!(\"Invalid jump destination\"),
            }
        }
    }
}",
    )?;

    Ok(())
}
