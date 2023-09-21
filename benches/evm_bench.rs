use bytes::Bytes;
use criterion::{criterion_group, criterion_main, Criterion};
use evm_dynamic::{fib, fib_repeated};
use jitevm::{ops_to_bytecode, run_jit_rust, run_revm_interpreter, test_data};

pub fn fib_benchmark(c: &mut Criterion) {
    let bytecode = ops_to_bytecode(test_data::get_code_ops_fibonacci());
    let calldata = Bytes::new();
    let jit_fn_unchecked = fib::<false>;
    let jit_fn_checked = fib::<true>;

    let mut group = c.benchmark_group("Fibonacci");

    group.bench_function("REVM Interpreter Fib", |b| {
        b.iter(|| run_revm_interpreter(&bytecode, &calldata))
    });
    // c.bench_function("jit_llvm_fib", |b| {
    //     b.iter(|| run_jit_evm(&bytecode, &calldata))
    // });
    group.bench_function("JIT Compiled Fib Checked", |b| {
        b.iter(|| run_jit_rust(&bytecode, &calldata, jit_fn_checked))
    });
    group.bench_function("JIT Compiled Fib Unchecked", |b| {
        b.iter(|| run_jit_rust(&bytecode, &calldata, jit_fn_unchecked))
    });
    group.finish();
}

pub fn fib_repeated_benchmark(c: &mut Criterion) {
    let bytecode = ops_to_bytecode(test_data::get_code_ops_fibonacci_repetitions());
    let calldata = Bytes::new();
    let jit_fn_checked = fib_repeated::<true>;
    let jit_fn_unchecked = fib_repeated::<false>;

    let mut group = c.benchmark_group("Fibonacci Repeated");

    group.bench_function("REVM Interpreter Fib Repeated", |b| {
        b.iter(|| run_revm_interpreter(&bytecode, &calldata))
    });
    // c.bench_function("jit_llvm_fib", |b| {
    //     b.iter(|| run_jit_evm(&bytecode, &calldata))
    // });
    group.bench_function("JIT Compiled Fib Repeated Checked", |b| {
        b.iter(|| run_jit_rust(&bytecode, &calldata, jit_fn_checked))
    });
    group.bench_function("JIT Compiled Fib Repeated Unchecked", |b| {
        b.iter(|| run_jit_rust(&bytecode, &calldata, jit_fn_unchecked))
    });
    group.finish();
}

criterion_group!(benches, fib_benchmark, fib_repeated_benchmark);
criterion_main!(benches);
