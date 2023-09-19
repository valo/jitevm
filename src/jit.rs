use inkwell::context::Context;
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;
use revm::primitives::keccak256;
use std::collections::HashMap;
use std::convert::From;
use thiserror::Error;
// use inkwell::execution_engine::JitFunction;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::targets::{InitializationConfig, Target};
use inkwell::IntPredicate;
// use inkwell::values::{FunctionValue, PointerValue, PhiValue, IntValue, BasicValue};
use crate::code::{EvmOp, IndexedEvmCode};
use crate::constants::EVM_STACK_ELEMENT_SIZE;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::types::IntType; //PointerType};
use inkwell::values::{IntValue, PhiValue}; //PointerValue

use revm::primitives::U256;

pub type JitEvmCompiledContract = unsafe extern "C" fn(usize) -> u64;
const _EVM_JIT_STACK_ALIGN: u32 = 16;

macro_rules! op1_llvmnativei256_operation {
    ($self:ident, $book:ident, $fname:ident) => {{
        let (book, a) = $self.build_stack_pop($book);
        let d = $self.builder.$fname(a, "");
        let book = $self.build_stack_push(book, d);
        book
    }};
}

macro_rules! op2_llvmnativei256_operation {
    ($self:ident, $book:ident, $fname:ident) => {{
        let (book, a) = $self.build_stack_pop($book);
        let (book, b) = $self.build_stack_pop(book);
        let d = $self.builder.$fname(a, b, "");
        let book = $self.build_stack_push(book, d);
        book
    }};
}

macro_rules! op2_llvmnativei256_compare_operation {
    ($self:ident, $book:ident, $this:expr, $next:expr, $instructions:expr, $i:expr, $op:expr, $predicate:expr) => {{
        let (book, a) = $self.build_stack_pop($book);
        let (book, b) = $self.build_stack_pop(book);
        let cmp = $self.builder.build_int_compare($predicate, a, b, "");

        let push_0 = JitEvmEngineSimpleBlock::new(
            $self,
            $instructions[$i].block,
            &format!("Instruction #{}: {:?} / push 0", $i, $op),
            &format!("_{}_0", $i),
        );
        let push_1 = JitEvmEngineSimpleBlock::new(
            $self,
            push_0.block,
            &format!("Instruction #{}: {:?} / push 1", $i, $op),
            &format!("_{}_1", $i),
        );

        $self.builder.position_at_end($this.block);
        $self
            .builder
            .build_conditional_branch(cmp, push_1.block, push_0.block);
        push_0.add_incoming(&book, &$this);
        push_1.add_incoming(&book, &$this);

        $self.builder.position_at_end(push_0.block);
        let book_0 = $self.build_stack_push(book, $self.type_stackel.const_int(0, false));
        $self.builder.build_unconditional_branch($next.block);
        $next.add_incoming(&book_0, &push_0);

        $self.builder.position_at_end(push_1.block);
        let book_1 = $self.build_stack_push(book, $self.type_stackel.const_int(1, false));
        $self.builder.build_unconditional_branch($next.block);
        $next.add_incoming(&book_1, &push_1);

        continue; // skip auto-generated jump to next instruction
    }};
}

#[derive(Error, Debug)]
pub enum JitEvmEngineError {
    #[error("FunctionLookupError: {0:?}")]
    FunctionLookupError(#[from] inkwell::execution_engine::FunctionLookupError),
    #[error("LlvmStringError: {0:?}")]
    UnknownLlvmStringError(#[from] inkwell::support::LLVMString),
    #[error("StringError: {0:?}")]
    UnknownStringError(String),
}

impl From<String> for JitEvmEngineError {
    fn from(e: String) -> Self {
        Self::UnknownStringError(e)
    }
}

impl From<&str> for JitEvmEngineError {
    fn from(e: &str) -> Self {
        Self::UnknownStringError(e.to_string())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct JitEvmEngineBookkeeping<'ctx> {
    pub context: IntValue<'ctx>,
    pub sp: IntValue<'ctx>,
    pub mem: IntValue<'ctx>,
    // pub retval: IntValue<'ctx>,
}

impl<'ctx> JitEvmEngineBookkeeping<'ctx> {
    pub fn update_sp(&self, sp: IntValue<'ctx>) -> Self {
        Self { sp, ..*self }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct JitEvmEngineSimpleBlock<'ctx> {
    pub block: BasicBlock<'ctx>,
    pub phi_context: PhiValue<'ctx>,
    pub phi_sp: PhiValue<'ctx>,
    pub phi_mem: PhiValue<'ctx>,
}

impl<'ctx> JitEvmEngineSimpleBlock<'ctx> {
    pub fn new(
        engine: &JitEvmEngine<'ctx>,
        block_before: BasicBlock<'ctx>,
        name: &str,
        suffix: &str,
    ) -> Self {
        let i64_type = engine.context.i64_type();

        let block = engine.context.insert_basic_block_after(block_before, name);
        engine.builder.position_at_end(block);
        let phi_context = engine
            .builder
            .build_phi(i64_type, &format!("context{}", suffix));
        let phi_sp = engine.builder.build_phi(i64_type, &format!("sp{}", suffix));
        let phi_mem_start = engine
            .builder
            .build_phi(i64_type, &format!("mem_start{}", suffix));

        Self {
            block,
            phi_context,
            phi_sp,
            phi_mem: phi_mem_start,
        }
    }

    pub fn add_incoming(
        &self,
        book: &JitEvmEngineBookkeeping<'ctx>,
        prev: &JitEvmEngineSimpleBlock<'ctx>,
    ) {
        self.phi_context
            .add_incoming(&[(&book.context, prev.block)]);
        self.phi_sp.add_incoming(&[(&book.sp, prev.block)]);
        self.phi_mem.add_incoming(&[(&book.mem, prev.block)]);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JitEvmExecutionContext {
    // WARNING: if you change anything here (adding fields is ok), then you need to change:
    //           - LLVM instructions in "setup" block of "executecontract" function
    //           - JitEvmEngine::callback_sload, JitEvmEngine::callback_sstore, ...
    //           - possibly other code! => try not to change this!
    // TODO: these are really all pointers
    pub stack: usize,
    pub memory: usize,
    pub storage: usize,
}

impl JitEvmExecutionContext {
    pub fn new_from_holder(container: &mut JitEvmExecutionContextHolder) -> Self {
        Self {
            stack: &mut container.stack as *mut _ as usize,
            memory: &mut container.memory as *mut _ as usize,
            storage: &mut container.storage as *mut _ as usize,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JitEvmExecutionContextHolder {
    pub stack: [U256; 1024],
    pub memory: [u8; 4096000],
    pub storage: HashMap<U256, U256>,
}

impl JitEvmExecutionContextHolder {
    pub fn new_from_empty() -> Self {
        Self {
            stack: [U256::from(0); 1024],
            memory: [0u8; 4096000],
            storage: HashMap::<U256, U256>::new(),
        }
    }
}

pub struct JitEvmEngine<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub execution_engine: ExecutionEngine<'ctx>,
    pub optimization_level: OptimizationLevel,
    pub type_ptrint: IntType<'ctx>,
    pub type_stackel: IntType<'ctx>,
    pub type_memel: IntType<'ctx>,
    pub type_retval: IntType<'ctx>,
}

impl<'ctx> JitEvmEngine<'ctx> {
    pub fn new_from_context(
        context: &'ctx Context,
        optimization_level: OptimizationLevel,
    ) -> Result<Self, JitEvmEngineError> {
        Target::initialize_native(&InitializationConfig::default())?;

        let module = context.create_module("jitevm");
        let builder = context.create_builder();
        let execution_engine = module.create_jit_execution_engine(optimization_level)?;

        let target_data = execution_engine.get_target_data();
        let type_ptrint = context.ptr_sized_int_type(&target_data, None); // type for pointers (stack pointer, host interaction)
                                                                          // ensure consistency btw Rust/LLVM definition of compiled contract function
        assert_eq!(type_ptrint.get_bit_width(), 64);
        assert_eq!(usize::BITS, 64);
        // TODO: the above assumes that pointers address memory byte-wise!

        let type_stackel = context.custom_width_int_type(256); // type for stack elements
        assert_eq!(type_stackel.get_bit_width(), 256);
        assert_eq!(
            type_stackel.get_bit_width() as u64,
            EVM_STACK_ELEMENT_SIZE * 8
        );

        let type_memel = context.custom_width_int_type(256); // type for memory elements
        assert_eq!(type_memel.get_bit_width(), 256);

        let type_retval = context.i64_type(); // type for return value
                                              // ensure consistency btw Rust/LLVM definition of compiled contract function
        assert_eq!(type_retval.get_bit_width(), 64);
        assert_eq!(u64::BITS, 64);

        Ok(Self {
            context,
            module,
            builder,
            execution_engine,
            optimization_level,
            type_ptrint,
            type_stackel,
            type_memel,
            type_retval,
        })
    }

    // HELPER FUNCTIONS

    fn build_stack_push<'a>(
        &'a self,
        book: JitEvmEngineBookkeeping<'a>,
        val: IntValue<'a>,
    ) -> JitEvmEngineBookkeeping<'a> {
        let sp_offset = self.type_ptrint.const_int(EVM_STACK_ELEMENT_SIZE, false);

        let sp_ptr = self.builder.build_int_to_ptr(
            book.sp,
            self.type_stackel.ptr_type(AddressSpace::Generic),
            "sp_ptr",
        );
        self.builder.build_store(sp_ptr, val);
        let sp = self.builder.build_int_add(book.sp, sp_offset, "sp_int");

        book.update_sp(sp)
    }

    fn build_stack_pop<'a>(
        &'a self,
        book: JitEvmEngineBookkeeping<'a>,
    ) -> (JitEvmEngineBookkeeping<'a>, IntValue<'a>) {
        let sp_offset = self.type_ptrint.const_int(EVM_STACK_ELEMENT_SIZE, false);

        let sp = self.builder.build_int_sub(book.sp, sp_offset, "sp");
        let sp_ptr = self.builder.build_int_to_ptr(
            sp,
            self.type_stackel.ptr_type(AddressSpace::Generic),
            "sp_ptr",
        );
        let val = self.builder.build_load(sp_ptr, "val").into_int_value();

        (book.update_sp(sp), val)
    }

    fn build_stack_write<'a>(
        &'a self,
        book: JitEvmEngineBookkeeping<'a>,
        idx: u64,
        val: IntValue<'a>,
    ) -> JitEvmEngineBookkeeping<'a> {
        let idx = self
            .type_ptrint
            .const_int(idx * EVM_STACK_ELEMENT_SIZE, false);

        let sp_int = self.builder.build_int_sub(book.sp, idx, "sp_int");
        let sp_ptr = self.builder.build_int_to_ptr(
            sp_int,
            self.type_stackel.ptr_type(AddressSpace::Generic),
            "sp_ptr",
        );
        self.builder.build_store(sp_ptr, val);

        book
    }

    fn build_stack_read<'a>(
        &'a self,
        book: JitEvmEngineBookkeeping<'a>,
        idx: u64,
    ) -> (JitEvmEngineBookkeeping<'a>, IntValue<'a>) {
        let idx = self
            .type_ptrint
            .const_int(idx * EVM_STACK_ELEMENT_SIZE, false);

        let sp_int = self.builder.build_int_sub(book.sp, idx, "sp_int");
        let sp_ptr = self.builder.build_int_to_ptr(
            sp_int,
            self.type_stackel.ptr_type(AddressSpace::Generic),
            "sp_ptr",
        );
        let val = self.builder.build_load(sp_ptr, "val").into_int_value();

        (book, val)
    }

    fn build_memory_read<'a>(
        &'a self,
        book: JitEvmEngineBookkeeping<'a>,
        idx: IntValue<'a>,
    ) -> (JitEvmEngineBookkeeping<'a>, IntValue<'a>) {
        let index = self.builder.build_int_cast(idx, self.type_ptrint, "index");
        let mem_location = self.builder.build_int_add(book.mem, index, "mem_location");
        let mem_ptr = self.builder.build_int_to_ptr(
            mem_location,
            self.type_memel.ptr_type(AddressSpace::Generic),
            "mem_ptr",
        );
        let val = self.builder.build_load(mem_ptr, "val").into_int_value();

        (book, val)
    }

    fn build_memory_write<'a>(
        &'a self,
        book: JitEvmEngineBookkeeping<'a>,
        idx: IntValue<'a>,
        val: IntValue<'a>,
    ) -> JitEvmEngineBookkeeping<'a> {
        let index = self.builder.build_int_cast(idx, self.type_ptrint, "index");
        let mem_location = self.builder.build_int_add(book.mem, index, "mem_location");
        let mem_ptr = self.builder.build_int_to_ptr(
            mem_location,
            self.type_memel.ptr_type(AddressSpace::Generic),
            "mem_ptr",
        );
        self.builder.build_store(mem_ptr, val);

        book
    }

    fn build_dup<'a>(
        &'a self,
        book: JitEvmEngineBookkeeping<'a>,
        idx: u64,
    ) -> Result<JitEvmEngineBookkeeping<'a>, JitEvmEngineError> {
        let len_stackel = self.type_ptrint.const_int(EVM_STACK_ELEMENT_SIZE, false);
        let sp_src_offset = self
            .type_ptrint
            .const_int(idx * EVM_STACK_ELEMENT_SIZE, false);
        let src_int = self.builder.build_int_sub(book.sp, sp_src_offset, "");
        let src_ptr = self.builder.build_int_to_ptr(
            src_int,
            self.type_stackel.ptr_type(AddressSpace::Generic),
            "",
        );
        let dst_ptr = self.builder.build_int_to_ptr(
            book.sp,
            self.type_stackel.ptr_type(AddressSpace::Generic),
            "",
        );
        self.builder.build_memcpy(
            dst_ptr,
            _EVM_JIT_STACK_ALIGN,
            src_ptr,
            _EVM_JIT_STACK_ALIGN,
            len_stackel,
        )?;
        let sp = self.builder.build_int_add(book.sp, len_stackel, "");

        Ok(book.update_sp(sp))
    }

    fn build_swap<'a>(
        &'a self,
        book: JitEvmEngineBookkeeping<'a>,
        idx: u64,
    ) -> JitEvmEngineBookkeeping<'a> {
        let (book, a) = self.build_stack_read(book, 1);
        let (book, b) = self.build_stack_read(book, idx);
        let book = self.build_stack_write(book, 1, b);
        let book = self.build_stack_write(book, idx, a);
        book
    }

    // CALLBACKS FOR OPERATIONS THAT CANNOT HAPPEN PURELY WITHIN THE EVM

    pub extern "C" fn callback_sload(exectx: usize, sp: usize) -> u64 {
        let exectx: &mut JitEvmExecutionContext = unsafe { &mut *(exectx as *mut _) };
        let storage: &mut HashMap<U256, U256> = unsafe { &mut *(exectx.storage as *mut _) };

        let key: &mut U256 =
            unsafe { &mut *((sp - 1 * EVM_STACK_ELEMENT_SIZE as usize) as *mut _) };

        match storage.get(key) {
            Some(value) => {
                *key = *value;
            }
            None => {
                // TODO: proper error handling!
                panic!("Sload key not found: {}", *key);
            }
        }

        0
    }

    pub extern "C" fn callback_sstore(exectx: usize, sp: usize) -> u64 {
        let context: &mut JitEvmExecutionContext = unsafe { &mut *(exectx as *mut _) };
        let storage: &mut HashMap<U256, U256> = unsafe { &mut *(context.storage as *mut _) };

        let key: &mut U256 =
            unsafe { &mut *((sp - 1 * EVM_STACK_ELEMENT_SIZE as usize) as *mut _) };
        let value: &mut U256 =
            unsafe { &mut *((sp - 2 * EVM_STACK_ELEMENT_SIZE as usize) as *mut _) };

        storage.insert(*key, *value);

        0
    }

    pub extern "C" fn callback_exp(_exectx: usize, sp: usize) -> u64 {
        let base: &mut U256 =
            unsafe { &mut *((sp - 1 * EVM_STACK_ELEMENT_SIZE as usize) as *mut _) };
        let power: &mut U256 =
            unsafe { &mut *((sp - 2 * EVM_STACK_ELEMENT_SIZE as usize) as *mut _) };

        *power = base.overflowing_pow(*power).0;

        0
    }

    pub extern "C" fn callback_sha3(exectx: usize, sp: usize) -> u64 {
        let context: &mut JitEvmExecutionContext = unsafe { &mut *(exectx as *mut _) };
        let memory: &mut [u8; 4096000] = unsafe { &mut *(context.memory as *mut _) };

        let offset: &mut U256 =
            unsafe { &mut *((sp - 1 * EVM_STACK_ELEMENT_SIZE as usize) as *mut _) };
        let size: &mut U256 =
            unsafe { &mut *((sp - 2 * EVM_STACK_ELEMENT_SIZE as usize) as *mut _) };

        // TODO: Fail if these are not proper 64-bit values!
        let offset_int = offset.as_limbs()[0] as usize;
        let size_int = size.as_limbs()[0] as usize;

        let hash = keccak256(&memory[offset_int..offset_int + size_int]);

        *size = hash.into();

        0
    }

    // Allow to hook up instructions to Rust functions instead of implementing them in LLVM IR
    // This can be used for adding support for complex operations that cannot be implemented in LLVM IR
    //
    // pub extern "C" fn callback_add(ptr_a: usize, ptr_b: usize) -> u64 {
    //     let a: &mut U256 = unsafe { &mut *(ptr_a as *mut _) };
    //     let b: &mut U256 = unsafe { &mut *(ptr_b as *mut _) };
    //     // println!("In: {:?}, {:?}; Out: {:?}", *a, *b, operations::Add(*a, *b));
    //     *b = operations::Add(*a, *b);
    //     0
    // }

    pub fn jit_compile_contract(
        &self,
        code: IndexedEvmCode,
        debug_ir: Option<String>,
        debug_asm: Option<String>,
    ) -> Result<JitFunction<JitEvmCompiledContract>, JitEvmEngineError> {
        // CALLBACKS

        let callback_sload_func = {
            // SLOAD
            let cb_type = self
                .type_retval
                .fn_type(&[self.type_ptrint.into(), self.type_ptrint.into()], false);
            let cb_func = self.module.add_function("callback_sload", cb_type, None);
            self.execution_engine
                .add_global_mapping(&cb_func, JitEvmEngine::callback_sload as usize);
            cb_func
        };

        let callback_sstore_func = {
            // SSTORE
            let cb_type = self
                .type_retval
                .fn_type(&[self.type_ptrint.into(), self.type_ptrint.into()], false);
            let cb_func = self.module.add_function("callback_sstore", cb_type, None);
            self.execution_engine
                .add_global_mapping(&cb_func, JitEvmEngine::callback_sstore as usize);
            cb_func
        };

        let callback_exp_func = {
            // EXP
            let cb_type = self
                .type_retval
                .fn_type(&[self.type_ptrint.into(), self.type_ptrint.into()], false);
            let cb_func = self.module.add_function("callback_exp", cb_type, None);
            self.execution_engine
                .add_global_mapping(&cb_func, JitEvmEngine::callback_exp as usize);
            cb_func
        };

        let callback_sha3_func = {
            // SHA3
            let cb_type = self
                .type_retval
                .fn_type(&[self.type_ptrint.into(), self.type_ptrint.into()], false);
            let cb_func = self.module.add_function("callback_sha3", cb_type, None);
            self.execution_engine
                .add_global_mapping(&cb_func, JitEvmEngine::callback_sha3 as usize);
            cb_func
        };

        // let callback_add_func = { // ADD
        //     // let cb_type = self.type_stackel.fn_type(&[self.type_stackel.into(), self.type_stackel.into()], false);
        //     let cb_type = self.type_retval.fn_type(&[self.type_ptrint.into(), self.type_ptrint.into()], false);
        //     let cb_func = self.module.add_function("callback_add", cb_type, None);
        //     self.execution_engine.add_global_mapping(&cb_func, JitEvmEngine::callback_add as usize);
        //     cb_func
        // };

        // SETUP JIT'ED CONTRACT FUNCTION

        let executecontract_fn_type = self.type_retval.fn_type(&[self.type_ptrint.into()], false);
        let function = self
            .module
            .add_function("executecontract", executecontract_fn_type, None);

        // SETUP HANDLER

        let setup_block = self.context.append_basic_block(function, "setup");
        self.builder.position_at_end(setup_block);

        let setup_book = {
            let context_int = function.get_nth_param(0).unwrap().into_int_value();
            let context_ptr = self.builder.build_int_to_ptr(
                context_int,
                self.type_ptrint.ptr_type(AddressSpace::Generic),
                "context_ptr",
            );
            let stack_int = self
                .builder
                .build_load(context_ptr, "stack_int")
                .into_int_value();

            let mem_offset = self
                .type_ptrint
                .const_int(self.type_ptrint.get_bit_width() as u64 / 8, false);
            let mem_location = self
                .builder
                .build_int_add(context_int, mem_offset, "mem_location");
            let mem_ptr = self.builder.build_int_to_ptr(
                mem_location,
                self.type_ptrint.ptr_type(AddressSpace::Generic),
                "mem_ptr",
            );
            let mem_int = self.builder.build_load(mem_ptr, "mem_int").into_int_value();

            // let retval = self.type_retval.const_int(0, false);
            JitEvmEngineBookkeeping {
                context: context_int,
                sp: stack_int,
                mem: mem_int,
                // retval: retval
            }
        };

        // INSTRUCTIONS

        let ops_len = code.code.ops.len();
        assert!(ops_len > 0);

        let mut instructions: Vec<JitEvmEngineSimpleBlock<'_>> = Vec::new();
        for i in 0..ops_len {
            let block_before = if i == 0 {
                setup_block
            } else {
                instructions[i - 1].block
            };
            let label = format!("Instruction #{}: {:?}", i, code.code.ops[i]);
            instructions.push(JitEvmEngineSimpleBlock::new(
                self,
                block_before,
                &label,
                &format!("_{}", i),
            ));
        }

        self.builder.position_at_end(setup_block);
        self.builder
            .build_unconditional_branch(instructions[0].block);
        instructions[0]
            .phi_context
            .add_incoming(&[(&setup_book.context, setup_block)]);
        instructions[0]
            .phi_sp
            .add_incoming(&[(&setup_book.sp, setup_block)]);
        instructions[0]
            .phi_mem
            .add_incoming(&[(&setup_book.mem, setup_block)]);

        // END HANDLER

        let end =
            JitEvmEngineSimpleBlock::new(self, instructions[ops_len - 1].block, &"end", &"-end");
        self.builder
            .build_return(Some(&self.type_retval.const_int(0, false)));

        // ERROR-JUMPDEST HANDLER

        let error_jumpdest =
            JitEvmEngineSimpleBlock::new(self, end.block, &"error-jumpdest", &"-error-jumpdest");
        self.builder
            .build_return(Some(&self.type_retval.const_int(1, false)));

        // RENDER INSTRUCTIONS

        for (i, op) in code.code.ops.iter().enumerate() {
            use EvmOp::*;

            let this = instructions[i];

            self.builder.position_at_end(this.block);
            let book = JitEvmEngineBookkeeping {
                context: this.phi_context.as_basic_value().into_int_value(),
                sp: this.phi_sp.as_basic_value().into_int_value(),
                mem: this.phi_mem.as_basic_value().into_int_value(),
                // retval: this.phi_retval.as_basic_value().into_int_value(),
            };

            let next = if i + 1 == ops_len {
                end
            } else {
                instructions[i + 1]
            };

            let book = match op {
                Stop => {
                    let val = self.type_retval.const_int(0, false);
                    // TODO: change return value to be a pointer to memory, not a value
                    self.builder.build_return(Some(&val));
                    continue; // skip auto-generated jump to next instruction
                }
                Return => {
                    let (book, offset) = self.build_stack_pop(book);
                    let (book, _size) = self.build_stack_pop(book);
                    let (_book, value) = self.build_memory_read(book, offset);
                    let value_int = self
                        .builder
                        .build_int_cast(value, self.type_ptrint, "index");
                    // TODO: change return value to be a pointer to memory, not a value
                    self.builder.build_return(Some(&value_int));
                    continue;
                }
                Caller => {
                    // TODO: take the caller from the current context
                    let val = self.type_stackel.const_int(0, false);
                    let book = self.build_stack_push(book, val);
                    book
                }
                Callvalue => {
                    let val = self.type_stackel.const_int(0, false);
                    let book = self.build_stack_push(book, val);
                    book
                }
                Calldatasize => {
                    let val = self.type_stackel.const_int(0, false);
                    let book = self.build_stack_push(book, val);
                    book
                }
                Calldataload => {
                    let val = self.type_stackel.const_int(0, false);
                    let book = self.build_stack_push(book, val);
                    book
                }
                Revert => {
                    let (book, offset) = self.build_stack_pop(book);
                    let (book, _size) = self.build_stack_pop(book);
                    let (_book, value) = self.build_memory_read(book, offset);
                    let value_int = self
                        .builder
                        .build_int_cast(value, self.type_ptrint, "index");
                    // TODO: change return value to be a pointer to memory, not a value
                    self.builder.build_return(Some(&value_int));
                    continue;
                }
                Push(_, val) => {
                    let val = self
                        .type_stackel
                        .const_int_arbitrary_precision(val.as_limbs());
                    let book = self.build_stack_push(book, val);
                    book
                }
                Pop => {
                    let (book, _) = self.build_stack_pop(book);
                    book
                }
                Jumpdest => book,
                Sload => {
                    let _retval = self
                        .builder
                        .build_call(
                            callback_sload_func,
                            &[book.context.into(), book.sp.into()],
                            "",
                        )
                        .try_as_basic_value()
                        .left()
                        .unwrap()
                        .into_int_value();
                    // TODO: proper error handling, based on return value?
                    book
                }
                Sstore => {
                    let _retval = self
                        .builder
                        .build_call(
                            callback_sstore_func,
                            &[book.context.into(), book.sp.into()],
                            "",
                        )
                        .try_as_basic_value()
                        .left()
                        .unwrap()
                        .into_int_value();
                    // TODO: proper error handling, based on return value?
                    let (book, _) = self.build_stack_pop(book);
                    let (book, _) = self.build_stack_pop(book);
                    book
                }
                Mstore => {
                    let (book, idx) = self.build_stack_pop(book);
                    let (book, val) = self.build_stack_pop(book);
                    let book = self.build_memory_write(book, idx, val);

                    book
                }
                Mload => {
                    let (book, idx) = self.build_stack_pop(book);
                    let (book, val) = self.build_memory_read(book, idx);
                    let book = self.build_stack_push(book, val);

                    book
                }
                Sha3 => {
                    let _retval = self
                        .builder
                        .build_call(
                            callback_sha3_func,
                            &[book.context.into(), book.sp.into()],
                            "",
                        )
                        .try_as_basic_value()
                        .left()
                        .unwrap()
                        .into_int_value();
                    // The result will be in the stack at the second position, so we need to
                    // pop the base out
                    let (book, _) = self.build_stack_pop(book);
                    book
                }
                Log2 => {
                    // TODO: implement
                    book
                }
                Jump => {
                    let (book, target) = self.build_stack_pop(book);

                    if code.jumpdests.is_empty() {
                        // there are no valid jump targets, this Jump has to fail!
                        self.builder.build_unconditional_branch(end.block);
                        end.add_incoming(&book, &this);
                    } else {
                        let mut jump_table: Vec<JitEvmEngineSimpleBlock<'_>> = Vec::new();
                        for (j, jmp_i) in code.jumpdests.iter().enumerate() {
                            let jmp_target = code.opidx2target[jmp_i];
                            jump_table.push(JitEvmEngineSimpleBlock::new(
                                self,
                                if j == 0 {
                                    this.block
                                } else {
                                    jump_table[j - 1].block
                                },
                                &format!(
                                    "instruction #{}: {:?} / to Jumpdest #{} at op #{} to byte #{}",
                                    i, op, j, jmp_i, jmp_target
                                ),
                                &format!("_{}_{}", i, j),
                            ));
                        }

                        self.builder.position_at_end(this.block);
                        self.builder.build_unconditional_branch(jump_table[0].block);
                        jump_table[0].add_incoming(&book, &this);

                        for (j, jmp_i) in code.jumpdests.iter().enumerate() {
                            let jmp_target = code.opidx2target[jmp_i];
                            let jmp_target = jmp_target.try_into().unwrap(); // REMARK: assumes that code cannot exceed 2^64 instructions, probably ok ;)
                            self.builder.position_at_end(jump_table[j].block);
                            let cmp = self.builder.build_int_compare(
                                IntPredicate::EQ,
                                self.type_stackel.const_int(jmp_target, false),
                                target,
                                "",
                            );
                            if j + 1 == code.jumpdests.len() {
                                self.builder.build_conditional_branch(
                                    cmp,
                                    instructions[*jmp_i].block,
                                    error_jumpdest.block,
                                );
                                instructions[*jmp_i].add_incoming(&book, &jump_table[j]);
                                error_jumpdest.add_incoming(&book, &jump_table[j]);
                            } else {
                                self.builder.build_conditional_branch(
                                    cmp,
                                    instructions[*jmp_i].block,
                                    jump_table[j + 1].block,
                                );
                                instructions[*jmp_i].add_incoming(&book, &jump_table[j]);
                                jump_table[j + 1].add_incoming(&book, &jump_table[j]);
                            }
                        }
                    }

                    continue; // skip auto-generated jump to next instruction
                }
                Jumpi => {
                    let (book, target) = self.build_stack_pop(book);
                    let (book, val) = self.build_stack_pop(book);

                    if code.jumpdests.is_empty() {
                        // there are no valid jump targets, this Jumpi has to fail!
                        self.builder.build_unconditional_branch(end.block);
                        end.add_incoming(&book, &this);
                    } else {
                        let mut jump_table: Vec<JitEvmEngineSimpleBlock<'_>> = Vec::new();
                        for (j, jmp_i) in code.jumpdests.iter().enumerate() {
                            let jmp_target = code.opidx2target[jmp_i];
                            jump_table.push(JitEvmEngineSimpleBlock::new(
                                self,
                                if j == 0 {
                                    this.block
                                } else {
                                    jump_table[j - 1].block
                                },
                                &format!(
                                    "instruction #{}: {:?} / to Jumpdest #{} at op #{} to byte #{}",
                                    i, op, j, jmp_i, jmp_target
                                ),
                                &format!("_{}_{}", i, j),
                            ));
                        }

                        self.builder.position_at_end(this.block);
                        let cmp = self.builder.build_int_compare(
                            IntPredicate::EQ,
                            self.type_stackel.const_int(0, false),
                            val,
                            "",
                        );
                        self.builder
                            .build_conditional_branch(cmp, next.block, jump_table[0].block);
                        next.add_incoming(&book, &this);
                        jump_table[0].add_incoming(&book, &this);

                        for (j, jmp_i) in code.jumpdests.iter().enumerate() {
                            let jmp_target = code.opidx2target[jmp_i];
                            let jmp_target = jmp_target.try_into().unwrap(); // REMARK: assumes that code cannot exceed 2^64 instructions, probably ok ;)
                            self.builder.position_at_end(jump_table[j].block);
                            let cmp = self.builder.build_int_compare(
                                IntPredicate::EQ,
                                self.type_stackel.const_int(jmp_target, false),
                                target,
                                "",
                            );
                            if j + 1 == code.jumpdests.len() {
                                self.builder.build_conditional_branch(
                                    cmp,
                                    instructions[*jmp_i].block,
                                    error_jumpdest.block,
                                );
                                instructions[*jmp_i].add_incoming(&book, &jump_table[j]);
                                error_jumpdest.add_incoming(&book, &jump_table[j]);
                            } else {
                                self.builder.build_conditional_branch(
                                    cmp,
                                    instructions[*jmp_i].block,
                                    jump_table[j + 1].block,
                                );
                                instructions[*jmp_i].add_incoming(&book, &jump_table[j]);
                                jump_table[j + 1].add_incoming(&book, &jump_table[j]);
                            }
                        }
                    }

                    continue; // skip auto-generated jump to next instruction
                }
                Swap1 => self.build_swap(book, 1 + 1),
                Swap2 => self.build_swap(book, 2 + 1),
                Swap3 => self.build_swap(book, 3 + 1),
                Swap4 => self.build_swap(book, 4 + 1),
                Swap5 => self.build_swap(book, 5 + 1),
                Swap6 => self.build_swap(book, 6 + 1),
                Swap7 => self.build_swap(book, 7 + 1),
                Swap8 => self.build_swap(book, 8 + 1),
                Swap9 => self.build_swap(book, 9 + 1),
                Swap10 => self.build_swap(book, 10 + 1),
                Swap11 => self.build_swap(book, 11 + 1),
                Swap12 => self.build_swap(book, 12 + 1),
                Swap13 => self.build_swap(book, 13 + 1),
                Swap14 => self.build_swap(book, 14 + 1),
                Swap15 => self.build_swap(book, 15 + 1),
                Swap16 => self.build_swap(book, 16 + 1),
                Dup1 => self.build_dup(book, 1)?,
                Dup2 => self.build_dup(book, 2)?,
                Dup3 => self.build_dup(book, 3)?,
                Dup4 => self.build_dup(book, 4)?,
                Dup5 => self.build_dup(book, 5)?,
                Dup6 => self.build_dup(book, 6)?,
                Dup7 => self.build_dup(book, 7)?,
                Dup8 => self.build_dup(book, 8)?,
                Dup9 => self.build_dup(book, 9)?,
                Dup10 => self.build_dup(book, 10)?,
                Dup11 => self.build_dup(book, 11)?,
                Dup12 => self.build_dup(book, 12)?,
                Dup13 => self.build_dup(book, 13)?,
                Dup14 => self.build_dup(book, 14)?,
                Dup15 => self.build_dup(book, 15)?,
                Dup16 => self.build_dup(book, 16)?,
                Iszero => {
                    let (book, val) = self.build_stack_pop(book);
                    let cmp = self.builder.build_int_compare(
                        IntPredicate::EQ,
                        self.type_stackel.const_int(0, false),
                        val,
                        "",
                    );

                    let push_0 = JitEvmEngineSimpleBlock::new(
                        self,
                        instructions[i].block,
                        &format!("Instruction #{}: {:?} / push 0", i, op),
                        &format!("_{}_0", i),
                    );
                    let push_1 = JitEvmEngineSimpleBlock::new(
                        self,
                        push_0.block,
                        &format!("Instruction #{}: {:?} / push 1", i, op),
                        &format!("_{}_1", i),
                    );

                    self.builder.position_at_end(this.block);
                    self.builder
                        .build_conditional_branch(cmp, push_1.block, push_0.block);
                    push_0.add_incoming(&book, &this);
                    push_1.add_incoming(&book, &this);

                    self.builder.position_at_end(push_0.block);
                    let book_0 = self.build_stack_push(book, self.type_stackel.const_int(0, false));
                    self.builder.build_unconditional_branch(next.block);
                    next.add_incoming(&book_0, &push_0);

                    self.builder.position_at_end(push_1.block);
                    let book_1 = self.build_stack_push(book, self.type_stackel.const_int(1, false));
                    self.builder.build_unconditional_branch(next.block);
                    next.add_incoming(&book_1, &push_1);

                    continue; // skip auto-generated jump to next instruction
                }
                Add => {
                    op2_llvmnativei256_operation!(self, book, build_int_add)
                }
                Sub => {
                    op2_llvmnativei256_operation!(self, book, build_int_sub)
                }
                Mul => {
                    op2_llvmnativei256_operation!(self, book, build_int_mul)
                }
                Div => {
                    op2_llvmnativei256_operation!(self, book, build_int_unsigned_div)
                }
                Sdiv => {
                    op2_llvmnativei256_operation!(self, book, build_int_signed_div)
                }
                Mod => {
                    op2_llvmnativei256_operation!(self, book, build_int_unsigned_rem)
                }
                Exp => {
                    let _retval = self
                        .builder
                        .build_call(
                            callback_exp_func,
                            &[book.context.into(), book.sp.into()],
                            "",
                        )
                        .try_as_basic_value()
                        .left()
                        .unwrap()
                        .into_int_value();
                    // The result will be in the stack at the second position, so we need to
                    // pop the base out
                    let (book, _) = self.build_stack_pop(book);
                    book
                }
                // Smod => { op2_llvmnativei256_operation!(self, book, build_int_signed_rem) },
                Eq => {
                    op2_llvmnativei256_compare_operation!(
                        self,
                        book,
                        this,
                        next,
                        instructions,
                        i,
                        op,
                        IntPredicate::EQ
                    )
                }
                Lt => {
                    op2_llvmnativei256_compare_operation!(
                        self,
                        book,
                        this,
                        next,
                        instructions,
                        i,
                        op,
                        IntPredicate::ULT
                    )
                }
                Gt => {
                    op2_llvmnativei256_compare_operation!(
                        self,
                        book,
                        this,
                        next,
                        instructions,
                        i,
                        op,
                        IntPredicate::UGT
                    )
                }
                Slt => {
                    op2_llvmnativei256_compare_operation!(
                        self,
                        book,
                        this,
                        next,
                        instructions,
                        i,
                        op,
                        IntPredicate::SLT
                    )
                }
                Sgt => {
                    op2_llvmnativei256_compare_operation!(
                        self,
                        book,
                        this,
                        next,
                        instructions,
                        i,
                        op,
                        IntPredicate::SGT
                    )
                }
                And => {
                    op2_llvmnativei256_operation!(self, book, build_and)
                }
                Or => {
                    op2_llvmnativei256_operation!(self, book, build_or)
                }
                // Xor => { op2_llvmnativei256_operation!(self, book, build_xor) },
                Not => {
                    op1_llvmnativei256_operation!(self, book, build_not)
                }
                Shr => {
                    let (book, shift) = self.build_stack_pop(book);
                    let (book, value) = self.build_stack_pop(book);
                    let result = self.builder.build_right_shift(value, shift, false, "");
                    let book = self.build_stack_push(book, result);
                    book
                }
                Shl => {
                    let (book, shift) = self.build_stack_pop(book);
                    let (book, value) = self.build_stack_pop(book);
                    let result = self.builder.build_left_shift(value, shift, "");
                    let book = self.build_stack_push(book, result);
                    book
                }
                AugmentedPushJump(_, val) => {
                    if code.jumpdests.is_empty() {
                        // there are no valid jump targets, this Jump has to fail!
                        self.builder.build_unconditional_branch(end.block);
                        end.add_incoming(&book, &this);
                    } else {
                        // retrieve the corresponding jump target (panic if not a valid jump target) ...
                        let jmp_i = code.target2opidx[val];
                        // ... and jump to there!
                        self.builder
                            .build_unconditional_branch(instructions[jmp_i].block);
                        instructions[jmp_i].add_incoming(&book, &this);
                    }

                    continue; // skip auto-generated jump to next instruction
                }
                AugmentedPushJumpi(_, val) => {
                    let (book, condition) = self.build_stack_pop(book);

                    if code.jumpdests.is_empty() {
                        // there are no valid jump targets, this Jumpi has to fail!
                        self.builder.build_unconditional_branch(end.block);
                        end.add_incoming(&book, &this);
                    } else {
                        // retrieve the corresponding jump target (panic if not a valid jump target) ...
                        let jmp_i = code.target2opidx[val];
                        // ... and jump to there (conditionally)!
                        let cmp = self.builder.build_int_compare(
                            IntPredicate::EQ,
                            self.type_stackel.const_int(0, false),
                            condition,
                            "",
                        );
                        self.builder.build_conditional_branch(
                            cmp,
                            next.block,
                            instructions[jmp_i].block,
                        );
                        next.add_incoming(&book, &this);
                        instructions[jmp_i].add_incoming(&book, &this);
                    }

                    continue; // skip auto-generated jump to next instruction
                }

                Invalid => {
                    let value_int = self.type_ptrint.const_int(0, false);
                    self.builder.build_return(Some(&value_int));
                    continue;
                }

                Unknown(_op) => {
                    // TODO: Figure out what to do in these cases
                    book
                }

                _ => {
                    panic!("Op not implemented: {:?}", op);
                }
            };

            self.builder.build_unconditional_branch(next.block);
            next.add_incoming(&book, &this);
        }

        // OUTPUT LLVM
        if let Some(path) = debug_ir {
            self.module.print_to_file(path)?;
        }

        // OUTPUT ASM
        if let Some(path) = debug_asm {
            // https://github.com/TheDan64/inkwell/issues/184
            // https://thedan64.github.io/inkwell/inkwell/targets/struct.TargetMachine.html#method.write_to_file
            use inkwell::targets::{CodeModel, FileType, RelocMode, TargetMachine};

            let triple = TargetMachine::get_default_triple();
            let cpu = TargetMachine::get_host_cpu_name().to_string();
            let features = TargetMachine::get_host_cpu_features().to_string();

            let target = Target::from_triple(&triple).unwrap();
            let machine = target
                .create_target_machine(
                    &triple,
                    &cpu,
                    &features,
                    self.optimization_level,
                    RelocMode::Default,
                    CodeModel::Default,
                )
                .unwrap();

            // create a module and do JIT stuff
            machine.write_to_file(&self.module, FileType::Assembly, path.as_ref())?;
        }

        // COMPILE
        let run_fn: JitFunction<JitEvmCompiledContract> =
            unsafe { self.execution_engine.get_function("executecontract")? };
        Ok(run_fn)
    }
}
