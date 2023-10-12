use std::error::Error;

use bytes::Bytes;
use evm_dynamic::evm_cache::evm_cache;
use hex::encode;
use jitevm::{
    ops_to_bytecode, run_jit_rust, run_revm_interpreter,
    test_data::{self, get_code_bin_revm_test1},
};
use revm::{
    interpreter::{Host, Interpreter},
    primitives::{Bytecode, LatestSpec, B256},
};

fn main() -> Result<(), Box<dyn Error>> {
    let evm_code_cache = evm_cache::<true, LatestSpec>();
    let tests = vec![
        // (Name, Code, Call Data, AOT Function)
        (
            "Fibonacci".to_string(),
            ops_to_bytecode(test_data::get_code_ops_fibonacci()),
            Bytes::new(),
            evm_code_cache
                .get(
                    &"0x3bdc8674d4fde9f9dca23aa564ca243190a69977674148c40a8661f542582a4d"
                        .parse::<B256>()
                        .unwrap(),
                )
                .unwrap(),
        ),
        (
            "Fibonacci Repetitions".to_string(),
            ops_to_bytecode(test_data::get_code_ops_fibonacci_repetitions()),
            Bytes::new(),
            evm_code_cache
                .get(
                    &"0xbf10678ff0c6e3cae81657192f89f9b6c19df2bda65fcb80549cd8e5bd4ef911"
                        .parse::<B256>()
                        .unwrap(),
                )
                .unwrap(),
        ),
        // (
        //     "Snailtracer".to_string(),
        //     Bytes::from(get_code_bin_revm_test1()),
        //     Bytes::from(hex::decode("30627b7c").unwrap()),
        //     evm_code_cache
        //         .get(&"0xfea1a9".parse::<B256>().unwrap())
        //         .unwrap(),
        // ),
    ];

    println!("Fib bytecode: {}", hex::encode(&tests[1].1));

    for (name, bytecode, calldata, compiled_evm_code) in &tests {
        print!("Running test: {} ... ", name);
        // TESTING REVM INTERPRETER

        println!("Benchmarking interpreted execution ...");
        let (revm_runtime, revm_gas_used) = run_revm_interpreter(&bytecode, &calldata);
        println!("Runtime: {:.2?}", revm_runtime);
        println!("Gas used: {}", revm_gas_used);
        println!(
            "MGas/s: {:.2}",
            revm_gas_used as f64 / revm_runtime.as_secs_f64() / 1_000_000.0
        );

        // TESTING JIT

        // println!("Benchmarking JIT execution ...");
        // let jit_runtime = run_jit_evm(&bytecode, &calldata)?;
        // println!("Runtime: {:.2?}", jit_runtime);

        // println!(
        //     "Speedup: {:.2}x",
        //     revm_runtime.as_secs_f64() / jit_runtime.as_secs_f64()
        // );

        // TESTING RUST AOT COMPILATION

        println!("Benchmarking Rust AOT compilation ...");
        let (aot_runtime, aot_gas_used) = run_jit_rust(&bytecode, &calldata, compiled_evm_code);
        println!("Runtime: {:.2?}", aot_runtime);
        println!("Gas used: {}", aot_gas_used);
        println!(
            "MGas/s: {:.2}",
            aot_gas_used as f64 / aot_runtime.as_secs_f64() / 1_000_000.0
        );

        // assert!(revm_gas_used == (aot_gas_used + 21_000));

        // println!(
        //     "Speedup: {:.2}x",
        //     revm_runtime.as_secs_f64() / aot_runtime.as_secs_f64()
        // );
    }

    // let bytecode: Bytes = Bytes::from(get_code_bin_revm_test1());
    // generate_rust_code(&Bytecode::new_raw(bytecode.clone()))?;

    // let bytecode = tests[0].1.clone();
    // println!("Generating Rust code for: {:?}", encode(&bytecode));
    // generate_rust_code(&Bytecode::new_raw(bytecode.clone()))?;
    Ok(())
}
