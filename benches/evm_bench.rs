use bytes::Bytes;
use criterion::{criterion_group, criterion_main, Criterion};
use evm_dynamic::{
    evm_fe52880d7fca1f585e267c77d696523fb89925f31407bf97886a622217e1c3bd, fib, fib_repeated,
};
use jitevm::{ops_to_bytecode, run_jit_rust, run_revm_interpreter, test_data};
use revm::primitives::LatestSpec;

pub fn fib_benchmark(c: &mut Criterion) {
    let bytecode = ops_to_bytecode(test_data::get_code_ops_fibonacci());
    let calldata = Bytes::new();
    let jit_fn_unchecked = fib::call::<false>;
    let jit_fn_checked = fib::call::<true>;

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

pub fn snailtracer_benchmark(c: &mut Criterion) {
    let bytecode = Bytes::from(test_data::get_code_bin_revm_test1());
    let calldata = Bytes::from(hex::decode("30627b7c").unwrap());
    let jit_fn_checked = evm_fe52880d7fca1f585e267c77d696523fb89925f31407bf97886a622217e1c3bd::call::<
        true,
        LatestSpec,
    >;
    let jit_fn_unchecked =
        evm_fe52880d7fca1f585e267c77d696523fb89925f31407bf97886a622217e1c3bd::call::<
            false,
            LatestSpec,
        >;

    let mut group = c.benchmark_group("Snailtracer");

    group.bench_function("REVM Interpreter Snailtracer", |b| {
        b.iter(|| run_revm_interpreter(&bytecode, &calldata))
    });
    // c.bench_function("jit_llvm_fib", |b| {
    //     b.iter(|| run_jit_evm(&bytecode, &calldata))
    // });
    group.bench_function("JIT Compiled Fib Snailtracer Checked", |b| {
        b.iter(|| run_jit_rust(&bytecode, &calldata, jit_fn_checked))
    });
    group.bench_function("JIT Compiled Fib Snailtracer Unchecked", |b| {
        b.iter(|| run_jit_rust(&bytecode, &calldata, jit_fn_unchecked))
    });
    group.finish();
}

criterion_group!(
    benches,
    fib_benchmark,
    fib_repeated_benchmark,
    snailtracer_benchmark
);
criterion_main!(benches);
