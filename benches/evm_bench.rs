use bytes::Bytes;
use criterion::{criterion_group, criterion_main, Criterion};
use evm_dynamic::{fib, fib_repeated};
use jitevm::{ops_to_bytecode, run_jit_evm, run_jit_rust, run_revm_interpreter, test_data};

pub fn fib_benchmark(c: &mut Criterion) {
    let bytecode = ops_to_bytecode(test_data::get_code_ops_fibonacci());
    let calldata = Bytes::new();
    let jit_fn = fib;

    c.bench_function("revm_interpreter_fib", |b| {
        b.iter(|| run_revm_interpreter(&bytecode, &calldata))
    });
    // c.bench_function("jit_llvm_fib", |b| {
    //     b.iter(|| run_jit_evm(&bytecode, &calldata))
    // });
    c.bench_function("jit_rust_fib", |b| {
        b.iter(|| run_jit_rust(&bytecode, &calldata, jit_fn))
    });
}

pub fn fib_repeated_benchmark(c: &mut Criterion) {
    let bytecode = ops_to_bytecode(test_data::get_code_ops_fibonacci_repetitions());
    let calldata = Bytes::new();
    let jit_fn = fib_repeated;

    c.bench_function("revm_interpreter_fib_repeated", |b| {
        b.iter(|| run_revm_interpreter(&bytecode, &calldata))
    });
    // c.bench_function("jit_llvm_fib", |b| {
    //     b.iter(|| run_jit_evm(&bytecode, &calldata))
    // });
    c.bench_function("jit_rust_fib_repeated", |b| {
        b.iter(|| run_jit_rust(&bytecode, &calldata, jit_fn))
    });
}

criterion_group!(benches, fib_benchmark, fib_repeated_benchmark);
criterion_main!(benches);
