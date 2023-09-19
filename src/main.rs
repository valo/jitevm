use bytes::Bytes;
use eyre::Result;
use hex::encode;
use inkwell::context::Context;
use jitevm::code::{EvmCode, EvmOpParserMode};
use jitevm::jit::{JitEvmEngine, JitEvmEngineError, JitEvmExecutionContext};
use jitevm::test_data;
use primitive_types::U256;
use revm::db::states::plain_account::PlainStorage;
use revm::db::states::State;
use revm::primitives::ruint::Uint;
use revm::primitives::{keccak256, Bytecode, Env, SpecId, B160, B256};

use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, Instant};

fn run_jit_evm(code: &Bytes, calldata: &Bytes) -> Result<Duration, JitEvmEngineError> {
    let context = Context::create();
    let optimization_level = inkwell::OptimizationLevel::Aggressive;
    let engine = JitEvmEngine::new_from_context(&context, optimization_level)
        .expect("Failed to create engine");

    let ops = EvmCode::new_from_bytes(&code, EvmOpParserMode::Lax)
        .unwrap()
        .ops;
    let code = EvmCode { ops: ops };
    let augmented_code = code.augment().index();
    let contract = engine.jit_compile_contract(
        augmented_code,
        Some("jit_main.ll".to_string()),
        Some("jit_main.asm".to_string()),
    )?;

    println!("Benchmark compiled execution ...");

    let mut execution_context_stack = [U256::zero(); 1024];
    // TODO: at maximum block size of 30M gas, max memory size is 123169 words = ~128000 words = 4096000 bytes
    let mut execution_context_memory = [0u8; 4096000];
    let mut execution_context_storage = HashMap::<U256, U256>::new();

    let mut execution_context = JitEvmExecutionContext {
        stack: &mut execution_context_stack as *mut _ as usize,
        memory: &mut execution_context_memory as *mut _ as usize,
        storage: &mut execution_context_storage as *mut _ as usize,
    };
    // println!("INPUT: {:?}", execution_context.clone());

    let context_ptr = &mut execution_context as *mut _ as usize;
    // println!("Context ptr: {:x}", context_ptr);
    // println!("Stack ptr: {:x}", execution_context.stack);
    // println!("Memory ptr: {:x}", execution_context.memory);

    let measurement_now = Instant::now();

    let ret = unsafe { contract.call(context_ptr) };

    println!("Ret: {:?}", ret);
    // println!("Stack: {:?}", execution_context_stack);

    let llvm_execution = measurement_now.elapsed();

    Ok(llvm_execution)
}

fn run_revm_interpreter(code: &Bytes, calldata: &Bytes) -> Duration {
    // println!("Code: {:?}", encode(&code));
    let mut env = Env::default();
    // cfg env. SpecId is set down the road
    env.cfg.chain_id = 1; // for mainnet

    // block env
    env.block.number = Uint::from(1);
    env.block.coinbase = B160::from(0);
    env.block.timestamp = Uint::from(0);
    env.block.gas_limit = Uint::from(500_000_000);
    env.block.basefee = Uint::from(0);
    env.block.difficulty = Uint::from(0);
    // after the Merge prevrandao replaces mix_hash field in block and replaced difficulty opcode in EVM.
    env.block.prevrandao = Some(B256::from(B160::from(0)));

    //tx env
    env.tx.caller = B160::from(1);
    env.tx.gas_price = Uint::from(0);
    env.tx.gas_priority_fee = Some(Uint::from(0));
    env.tx.gas_limit = 500_000_000;
    env.cfg.spec_id = SpecId::LATEST;
    env.tx.data = calldata.clone();
    env.tx.value = Uint::from(0);
    env.tx.transact_to = revm::primitives::TransactTo::Call(B160::from(0));

    let mut state = State::builder().with_bundle_update().build();
    let acc_info = revm::primitives::AccountInfo {
        balance: Uint::from(0),
        code_hash: keccak256(&code),
        code: Some(Bytecode::new_raw(code.clone())),
        nonce: 0,
    };
    state.insert_account_with_storage(B160::from(0), acc_info, PlainStorage::new());

    let mut evm = revm::new();
    evm.database(&mut state);
    evm.env = env.clone();

    let timer = Instant::now();

    let result = evm.transact_commit().unwrap();
    println!("Result: {:?}", result);
    println!("Success: {:?}", result.is_success());
    println!(
        "Result: {:?}",
        encode(result.output().unwrap_or(&Bytes::new()))
    );
    println!("Gas used: {:?}", result.gas_used());

    return timer.elapsed();
}

fn ops_to_bytecode(ops: Vec<jitevm::code::EvmOp>) -> Bytes {
    let code = EvmCode { ops: ops };
    let bytecode = Bytes::from_iter(code.to_bytes());
    return bytecode;
}

fn main() -> Result<(), Box<dyn Error>> {
    let tests: Vec<(Bytes, Bytes)> = vec![
        // (Code, Call Data)
        (
            ops_to_bytecode(test_data::get_code_ops_fibonacci()),
            Bytes::new(),
        ),
        (
            ops_to_bytecode(test_data::get_code_ops_fibonacci_repetitions()),
            Bytes::new(),
        ),
        (
            ops_to_bytecode(test_data::get_code_ops_supersimple1()),
            Bytes::new(),
        ),
        (
            ops_to_bytecode(test_data::get_code_ops_supersimple2()),
            Bytes::new(),
        ),
        (
            ops_to_bytecode(test_data::get_code_ops_storage1()),
            Bytes::new(),
        ),
        (
            ops_to_bytecode(test_data::get_code_ops_mstore_mload()),
            Bytes::new(),
        ),
        (
            Bytes::from_iter(test_data::get_code_bin_revm_test1()),
            Bytes::from(hex::decode("30627b7c").unwrap()),
        ),
    ];

    for (bytecode, calldata) in tests {
        // TESTING REVM INTERPRETER

        println!("Benchmarking interpreted execution ...");
        let revm_runtime = run_revm_interpreter(&bytecode, &calldata);
        println!("Runtime: {:.2?}", revm_runtime);

        // TESTING JIT

        println!("Benchmarking JIT execution ...");
        let jit_runtime = run_jit_evm(&bytecode, &calldata)?;
        println!("Runtime: {:.2?}", jit_runtime);

        println!(
            "Speedup: {:.2}x",
            revm_runtime.as_secs_f64() / jit_runtime.as_secs_f64()
        );
    }
    Ok(())
}
