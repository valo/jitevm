use std::error::Error;

use bytes::Bytes;
use evm_dynamic::{fib, fib_repeated};
use jitevm::{ops_to_bytecode, run_jit_evm, run_jit_rust, run_revm_interpreter, test_data};
use revm::interpreter::{Host, Interpreter};

fn main() -> Result<(), Box<dyn Error>> {
    let tests: Vec<(
        String,
        Bytes,
        Bytes,
        for<'a, 'b> fn(&'a mut Interpreter, &'b mut (dyn Host + 'b)),
    )> = vec![
        // (Name, Code, Call Data, AOT Function)
        (
            "Fibonacci".to_string(),
            ops_to_bytecode(test_data::get_code_ops_fibonacci()),
            Bytes::new(),
            fib,
        ),
        (
            "Fibonacci Repetitions".to_string(),
            ops_to_bytecode(test_data::get_code_ops_fibonacci_repetitions()),
            Bytes::new(),
            fib_repeated,
        ),
    ];

    for (name, bytecode, calldata, jit_fn) in tests {
        print!("Running test: {} ... ", name);
        // TESTING REVM INTERPRETER

        println!("Benchmarking interpreted execution ...");
        let (revm_runtime, revm_gas_used) = run_revm_interpreter(&bytecode, &calldata);
        println!("Runtime: {:.2?}", revm_runtime);

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
        let (aot_runtime, aot_gas_used) = run_jit_rust(&bytecode, &calldata, jit_fn);
        println!("Runtime: {:.2?}", aot_runtime);

        assert!(revm_gas_used == (aot_gas_used + 21_000));

        println!(
            "Speedup: {:.2}x",
            revm_runtime.as_secs_f64() / aot_runtime.as_secs_f64()
        );
    }
    Ok(())
}
