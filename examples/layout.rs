#![allow(unused)]
use faeyne_lang::ir::GlobalScope;
use codespan::Span;
use faeyne_lang::ir::LazyFunc;
use faeyne_lang::ir::FuncSig;
use faeyne_lang::ir::Block;
use faeyne_lang::ir::Statment;
use faeyne_lang::reporting::ErrList;
use faeyne_lang::ir::GcPointer;
use faeyne_lang::ir::GlobalFunc;
use faeyne_lang::ir::Func;

use faeyne_lang::ir::FunctionHandle;
use faeyne_lang::basic_ops::buildin_add;
use mem_viewer::_print_type_of;
use faeyne_lang::ir::{ValueRet,Value,LazyVal};
// use mem_viewer::*;   

macro_rules! view_mem {
    ($var: expr) => {
        // Get the alignment of the variable
        let var_alignment = std::mem::align_of_val(&$var);
        
        // Print metadata of var: var_name, size, type, alignment
        println!("Name: {}", stringify!($var));
        _print_type_of(&$var);
        println!("Addr: {:016x}", &$var as *const _ as *const u8 as usize);
        println!("Size: {} bytes", std::mem::size_of_val(&$var));
        println!("Alignment: {} bytes", var_alignment);

        if format!("{:016x}", &$var as *const _ as *const u8 as usize).contains("007f") {
            // Indicate that the address is likely on the stack
            println!("Aloc: Likely Stack");
        } else {
            println!("Aloc: Likely Heap");
        }

        let mut ptr: *const u8 = &$var as *const _ as *const u8;
        let end: *const u8 = unsafe { ptr.add(std::mem::size_of_val(&$var)) };
        let mut byte_index = 0;

        println!(" Byte | Align |  Dec  |    Bin   ");
        println!("-------------------------------");

        while ptr < end {
            let byte = unsafe { *ptr };

            // Calculate alignment based on byte offset relative to var_alignment
            let relative_alignment = var_alignment - (byte_index % var_alignment);

            println!(" {:04} |  {:4}  | {:03}  | {:08b} ", 
                     byte_index, relative_alignment, byte as u8, byte as u8);

            ptr = unsafe { ptr.add(1) };
            byte_index += 1;
        }

        println!();
    };
}



// fn main() {
//     use faeyne_lang::ir::{Value, LazyVal, ValueRet, GenericRet};
    
//     // Let's initialize some values for inspection:
//     // let value = Value::Float(f64::from_bits(0xFFFFFFFFFFFFFFFF)); // A simple integer value.
//     // let value = Value::Int(69);
//     let handle = FunctionHandle::FFI(buildin_add);
//     let value = Value::Func(handle.clone()) ;
//     let lazy_val = LazyVal::Terminal(value.clone()); // A LazyVal holding a terminal value.
//     let value_ret: ValueRet = GenericRet::new_local(value.clone()); // A ValueRet holding the same value.

//     println!("### Inspecting Handle ###");
//     view_mem!(handle);

//     // Use view_mem! macro to view the memory layout of each variable
//     println!("### Inspecting Value ###");
//     view_mem!(value);

//     println!("### Inspecting LazyVal ###");
//     view_mem!(lazy_val);

//     println!("### Inspecting ValueRet ###");
//     view_mem!(value_ret);
// }

pub type DynFFI<'ctx> = dyn Fn(Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList>;

#[derive(Clone)]
pub enum OtherHandle<'ctx> {
    FFI(fn(Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList>),
    // FFI2(fn(Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList>),
    // FFI3(fn(Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList>),
    // FFI4(fn(Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList>),
    // StateFFI(&'ctx DynFFI<'ctx>),
    StateFFI2(*const DynFFI<'ctx>),
    DataFFI(GcPointer<DynFFI<'ctx>>),
    // MutFFI(Box<dyn FnMut(Vec<Value>) -> Result<Value, ErrList>>), // New FnMut variant
    StaticDef(GlobalFunc<'ctx>),
    Lambda(GcPointer<Func<'ctx>>),
}

fn main() {
    let ffi_handle = OtherHandle::FFI(buildin_add);
    view_mem!(ffi_handle);

    let b = Box::new(|args: Vec<Value>| Ok(Value::Int(42)));
    view_mem!(b);

    let dyn_func = Box::leak(b);
    view_mem!(dyn_func);


    let state_ffi_handle =  OtherHandle::StateFFI2(dyn_func);
    view_mem!(state_ffi_handle);

}

// fn main() {


//     // let ffi_handle = FunctionHandle::FFI(buildin_add);
//     view_mem!(buildin_add);

//     let dyn_func = Box::leak(Box::new(|args: Vec<Value>| Ok(Value::Int(42))));
//     view_mem!(dyn_func);


//     let state_ffi_handle =  FunctionHandle::StateFFI(dyn_func);
//     view_mem!(state_ffi_handle);

//     let gc_func = GcPointer::new(|args: Vec<Value>| Ok(Value::Int(42)));
//     view_mem!(gc_func);
    
//     let data_ffi_handle = FunctionHandle::DataFFI(gc_func);
//     view_mem!(data_ffi_handle);


//     // Create a LazyFunc that modifies its own scope but should not modify the outer/global scope
//     let  sig = FuncSig { arg_ids: vec![2] };
//     let inner = Block::new(vec![
//             // Inside the function, we assign a new value to the same variable ID (1)
//             Statment::Assign(69, LazyVal::Terminal(Value::Int(200))),
//     ]);
//     let debug_span = Span::new(0,10);

//     let mut global_scope = Box::leak(Box::new(GlobalScope::default()));
//     global_scope.add(69, inner.clone(), sig.clone());

//     let lazy_func = LazyFunc{sig,inner,debug_span};
//     // view_mem!(lazy_func);

//     let func =lazy_func.eval(&global_scope.make_subscope(1).unwrap()).unwrap();
//     // view_mem!(func);

//     let static_def_handle = global_scope.get(69).unwrap();
//     // view_mem!(static_def_handle);

//     let Value::Func(FunctionHandle::StaticDef(global_func)) = static_def_handle.clone() else {panic!()};
//     view_mem!(global_func);


//     view_mem!(static_def_handle);

//     let lambda_handle = FunctionHandle::Lambda(GcPointer::new(func));
//     view_mem!(lambda_handle);


//     // Memory layout inspection using view_mem! macro

// }