pub mod code;
pub mod constants;
pub mod jit;
pub mod test_data;

use crate::code::{EvmCode, EvmOpParserMode};
use crate::jit::{JitEvmEngine, JitEvmEngineError, JitEvmExecutionContext};
use bytes::Bytes;
use eyre::Result;

use inkwell::context::Context;
use revm::db::states::plain_account::PlainStorage;
use revm::db::states::State;
use revm::db::EmptyDBTyped;
use revm::inspectors::NoOpInspector;

use revm::interpreter::{CallContext, CallScheme, Contract, Host, Interpreter};
use revm::precompile::Precompiles;
use revm::primitives::ruint::Uint;
use revm::primitives::{keccak256, Bytecode, Env, LatestSpec, SpecId, B160, B256, U256};
use revm::{to_precompile_id, EVMImpl};

use std::collections::HashMap;
use std::convert::Infallible;
use std::time::{Duration, Instant};

pub fn run_jit_evm(code: &Bytes, _calldata: &Bytes) -> Result<Duration, JitEvmEngineError> {
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

    // println!("Benchmark compiled execution ...");

    let mut execution_context_stack = [U256::from(0); 1024];
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

    // println!("Ret: {:?}", ret);
    // println!("Stack: {:?}", execution_context_stack);

    let llvm_execution = measurement_now.elapsed();

    Ok(llvm_execution)
}

pub fn run_revm_interpreter(code: &Bytes, calldata: &Bytes) -> (Duration, u64) {
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
    // println!("Result: {:?}", result.output().unwrap());
    // println!("Success: {:?}", result.is_success());
    // println!(
    //     "Result: {:?}",
    //     encode(result.output().unwrap_or(&Bytes::new()))
    // );
    // println!("Gas used: {:?}", result.gas_used());

    return (timer.elapsed(), result.gas_used());
}

pub fn run_jit_rust(
    code: &Bytes,
    calldata: &Bytes,
    f: for<'a, 'b> fn(&'a mut Interpreter, &'b mut (dyn Host + 'b)),
) -> (Duration, u64) {
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
    let code_hash = keccak256(&code);
    let acc_info = revm::primitives::AccountInfo {
        balance: Uint::from(0),
        code_hash: code_hash,
        code: Some(Bytecode::new_raw(code.clone())),
        nonce: 0,
    };
    state.insert_account_with_storage(B160::from(0), acc_info, PlainStorage::new());

    let mut evm = revm::new();
    evm.database(&mut state);
    evm.env = env.clone();

    let context = CallContext {
        caller: env.tx.caller,
        address: B160::from(0),
        code_address: B160::from(0),
        apparent_value: env.tx.value,
        scheme: CallScheme::Call,
    };

    let contract = Box::new(Contract::new_with_context(
        calldata.clone(),
        Bytecode::new_raw(code.clone()),
        code_hash,
        &context,
    ));

    let mut inspector = NoOpInspector;
    let mut host = Box::new(
        EVMImpl::<LatestSpec, State<EmptyDBTyped<Infallible>>, false>::new(
            &mut state,
            &mut env,
            &mut inspector,
            Precompiles::new(to_precompile_id(SpecId::LATEST)).clone(),
        ),
    );
    let mut interpreter = Box::new(Interpreter::new(contract, u64::MAX, true));

    let timer = Instant::now();

    f(interpreter.as_mut(), &mut *host);

    // println!("Result: {:?}", encode(interpreter.return_value()));
    // println!("Result: {:?}", interpreter.instruction_result);
    // println!("Gas used: {:?}", interpreter.gas.spend());

    return (timer.elapsed(), interpreter.gas.spend());
}

pub fn ops_to_bytecode(ops: Vec<crate::code::EvmOp>) -> Bytes {
    let code = EvmCode { ops: ops };
    let bytecode = Bytes::from_iter(code.to_bytes());

    return bytecode;
}
