#![allow(unused)]
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



fn main() {
    use faeyne_lang::ir::{Value, LazyVal, ValueRet, GenericRet};
    
    // Let's initialize some values for inspection:
    // let value = Value::Float(f64::from_bits(0xFFFFFFFFFFFFFFFF)); // A simple integer value.
    let value = Value::Int(69);
    let lazy_val = LazyVal::Terminal(value.clone()); // A LazyVal holding a terminal value.
    let value_ret: ValueRet = GenericRet::Local(value.clone()); // A ValueRet holding the same value.

    // Use view_mem! macro to view the memory layout of each variable
    println!("### Inspecting Value ###");
    view_mem!(value);

    println!("### Inspecting LazyVal ###");
    view_mem!(lazy_val);

    println!("### Inspecting ValueRet ###");
    view_mem!(value_ret);
}
