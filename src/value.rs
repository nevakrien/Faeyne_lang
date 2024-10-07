use crate::stack::{Aligned, Stack};
use ast::ast::StringTable;

// Enum for value types
#[repr(u32)] //used because the stack is 64bits aligned and this lets us cram a u32 id next to this baby
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueType {
    //0 is undefined
    Nil = 1,
    BoolTrue,
    BoolFalse,
    Atom,
    String,
    Int,
    Float,
    Func,
}

impl TryFrom<u32> for ValueType {
    type Error = ();
    #[inline(always)]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ValueType::Nil),
            2 => Ok(ValueType::BoolTrue),
            3 => Ok(ValueType::BoolFalse),
            4 => Ok(ValueType::Atom),
            5 => Ok(ValueType::String),
            6 => Ok(ValueType::Int),
            7 => Ok(ValueType::Float),
            8 => Ok(ValueType::Func),
            _ => Err(()),
        }
    }
}

#[repr(u32)] //this is to fit nicely with ValueType
#[derive(Debug, Clone, PartialEq)]
pub enum IRValue {
    Nil = ValueType::Nil as u32,
    Bool(bool),
    Atom(u32),
    String(u32),
    Int(i64),
    Float(f64),
    Func(u32),
}

pub struct Context<'ctx,'code,const STACK_SIZE: usize>{
    pub table: &'ctx StringTable<'code>,
    pub stack: &'ctx mut Stack,
}

// Trait for specialized Stack operations for IRValue
pub trait ValueStack {
    //note that push_grow is 2x slower than push. so its recommended to ensure capacity and then push with unwraps.
    fn pop_value(&mut self) -> Result<IRValue, ()>;
    fn pop_nil(&mut self) -> Result<(), ()>;
    fn pop_atom(&mut self) -> Result<u32, ()>;
    fn pop_string(&mut self) -> Result<u32, ()>;
    fn pop_int(&mut self) -> Result<i64, ()>;
    fn pop_float(&mut self) -> Result<f64, ()>;
    
    fn push_value(&mut self, value: &IRValue) -> Result<(), ()>;    
    fn push_nil(&mut self) -> Result<(), ()>;
    fn push_bool(&mut self, val: bool) -> Result<(), ()>;
    fn push_atom(&mut self, id: u32) -> Result<(), ()>;
    fn push_string(&mut self, id: u32) -> Result<(), ()>;
    fn push_int(&mut self, val: i64) -> Result<(), ()>;
    fn push_float(&mut self, val: f64) -> Result<(), ()>;
    fn push_func(&mut self, id: u32) -> Result<(), ()>;

    fn push_grow_value(&mut self, value: &IRValue);    
    fn push_grow_nil(&mut self);
    fn push_grow_bool(&mut self, val: bool);
    fn push_grow_atom(&mut self, id: u32);
    fn push_grow_string(&mut self, id: u32);
    fn push_grow_int(&mut self, val: i64);
    fn push_grow_float(&mut self, val: f64);
    fn push_grow_func(&mut self, id: u32);
}

impl ValueStack for Stack {
    #[inline]
    fn push_value(&mut self, value: &IRValue) -> Result<(), ()> {
        match value {
            IRValue::Nil => self.push_nil(),
            IRValue::Bool(val) => self.push_bool(*val),
            IRValue::Atom(id) => self.push_atom(*id),
            IRValue::String(id) => self.push_string(*id),
            IRValue::Int(val) => self.push_int(*val),
            IRValue::Float(val) => self.push_float(*val),
            IRValue::Func(id) => self.push_func(*id),
        }
    }

    #[inline]
    fn push_grow_value(&mut self, value: &IRValue){
        match value {
            IRValue::Nil => self.push_grow_nil(),
            IRValue::Bool(val) => self.push_grow_bool(*val),
            IRValue::Atom(id) => self.push_grow_atom(*id),
            IRValue::String(id) => self.push_grow_string(*id),
            IRValue::Int(val) => self.push_grow_int(*val),
            IRValue::Float(val) => self.push_grow_float(*val),
            IRValue::Func(id) => self.push_grow_func(*id),
        }
    }


    #[inline]
    fn pop_value(&mut self) -> Result<IRValue, ()> {
        unsafe {
            match self.pop::<u64>() {
                Some(aligned_tag) => {
                    let tag = aligned_tag.to_inner();
                    match ValueType::try_from((tag & 0xFFFFFFFF) as u32) {
                        Ok(ValueType::Nil) => Ok(IRValue::Nil),
                        Ok(ValueType::BoolTrue) => Ok(IRValue::Bool(true)),
                        Ok(ValueType::BoolFalse) => Ok(IRValue::Bool(false)),
                        Ok(ValueType::Atom) => Ok(IRValue::Atom((tag >> 32) as u32)),
                        Ok(ValueType::String) => Ok(IRValue::String((tag >> 32) as u32)),
                        Ok(ValueType::Func) => Ok(IRValue::Func((tag >> 32) as u32)),
                        Ok(ValueType::Int) => {
                            let data = match self.pop::<u64>() {
                                Some(aligned_data) => aligned_data,
                                None => return Err(()),
                            };
                            Ok(IRValue::Int(data.to_inner() as i64))
                        }
                        Ok(ValueType::Float) => {
                            let data = match self.pop::<u64>() {
                                Some(aligned_data) => aligned_data,
                                None => return Err(()),
                            };
                            Ok(IRValue::Float(f64::from_bits(data.to_inner())))
                        }
                        _ => Err(()),
                    }
                }
                None => Err(()),
            }
        }
    }

    #[inline]
    fn pop_nil(&mut self) -> Result<(), ()> {
        if let Some(aligned) = unsafe { self.pop::<u64>() } {
            let tag = aligned.to_inner();
            if tag == ValueType::Nil as u64 {
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    #[inline]
    fn pop_atom(&mut self) -> Result<u32, ()> {
        if let Some(aligned) = unsafe { self.pop::<u64>() } {
            let tag = aligned.to_inner();
            if (tag & 0xFFFFFFFF) as u32 == ValueType::Atom as u32 {
                Ok((tag >> 32) as u32)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    #[inline]
    fn pop_string(&mut self) -> Result<u32, ()> {
        if let Some(aligned) = unsafe { self.pop::<u64>() } {
            let tag = aligned.to_inner();
            if (tag & 0xFFFFFFFF) as u32 == ValueType::String as u32 {
                Ok((tag >> 32) as u32)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    #[inline]
    fn pop_int(&mut self) -> Result<i64, ()> {
        if let Some(aligned_tag) = unsafe { self.pop::<u64>() } {
            let tag = aligned_tag.to_inner();
            if (tag & 0xFFFFFFFF) as u32 == ValueType::Int as u32 {
                if let Some(aligned_data) = unsafe { self.pop::<u64>() } {
                    Ok(aligned_data.to_inner() as i64)
                } else {
                    Err(())
                }
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    #[inline]
    fn pop_float(&mut self) -> Result<f64, ()> {
        if let (Some(aligned_tag), Some(aligned_data)) = (unsafe { self.pop::<u64>() }, unsafe { self.pop::<u64>() }) {
            let tag = aligned_tag.to_inner();
            if (tag & 0xFFFFFFFF) as u32 == ValueType::Float as u32 {
                Ok(f64::from_bits(aligned_data.to_inner()))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    #[inline]
    fn push_nil(&mut self) -> Result<(), ()> {
        let tag = ValueType::Nil as u64;
        let aligned = Aligned::new(tag);
        self.push(&aligned)
    }

    #[inline]
    fn push_grow_nil(&mut self) {
        let tag = ValueType::Nil as u64;
        let aligned = Aligned::new(tag);
        self.push_grow(&aligned)
    }

    #[inline]
    fn push_bool(&mut self, val: bool) -> Result<(), ()> {
        let tag = if val {
            ValueType::BoolTrue as u64
        } else {
            ValueType::BoolFalse as u64
        };
        let aligned = Aligned::new(tag);
        self.push(&aligned)
    }

    #[inline]
    fn push_grow_bool(&mut self, val: bool)  {
        let tag = if val {
            ValueType::BoolTrue as u64
        } else {
            ValueType::BoolFalse as u64
        };
        let aligned = Aligned::new(tag);
        self.push_grow(&aligned)
    }

    #[inline]
    fn push_atom(&mut self, id: u32) -> Result<(), ()> {
        let packed_data = (id as u64) << 32 | ValueType::Atom as u64;
        let aligned = Aligned::new(packed_data);
        self.push(&aligned)
    }

    #[inline]
    fn push_grow_atom(&mut self, id: u32) {
        let packed_data = (id as u64) << 32 | ValueType::Atom as u64;
        let aligned = Aligned::new(packed_data);
        self.push_grow(&aligned)
    }

    #[inline]
    fn push_string(&mut self, id: u32) -> Result<(), ()> {
        let packed_data = (id as u64) << 32 | ValueType::String as u64;
        let aligned = Aligned::new(packed_data);
        self.push(&aligned)
    }

    #[inline]
    fn push_grow_string(&mut self, id: u32) {
        let packed_data = (id as u64) << 32 | ValueType::String as u64;
        let aligned = Aligned::new(packed_data);
        self.push_grow(&aligned)
    }

    #[inline]
    fn push_int(&mut self, val: i64) -> Result<(), ()> {
        let tag = ValueType::Int as u64;
        let aligned_data = Aligned::new(val as u64);
        let aligned_tag = Aligned::new(tag);
        self.push(&aligned_data)?;
        self.push(&aligned_tag)
    }

    #[inline]
    fn push_grow_int(&mut self, val: i64)  {
        let tag = ValueType::Int as u64;
        let aligned_data = Aligned::new(val as u64);
        let aligned_tag = Aligned::new(tag);
        self.push_grow(&aligned_data);
        self.push_grow(&aligned_tag);
    }

    #[inline]
    fn push_float(&mut self, val: f64) -> Result<(), ()> {
        let tag = ValueType::Float as u64;
        let aligned_data = Aligned::new(val.to_bits() as u64);
        let aligned_tag = Aligned::new(tag);
        self.push(&aligned_data)?;
        self.push(&aligned_tag)
    }

    #[inline]
    fn push_grow_float(&mut self, val: f64) {
        let tag = ValueType::Float as u64;
        let aligned_data = Aligned::new(val.to_bits() as u64);
        let aligned_tag = Aligned::new(tag);
        self.push_grow(&aligned_data);
        self.push_grow(&aligned_tag);
    }

    #[inline]
    fn push_func(&mut self, id: u32) -> Result<(), ()> {
        let packed_data = (id as u64) << 32 | ValueType::Func as u64;
        let aligned = Aligned::new(packed_data);
        self.push(&aligned)
    }

    #[inline]
    fn push_grow_func(&mut self, id: u32) {
        let packed_data = (id as u64) << 32 | ValueType::Func as u64;
        let aligned = Aligned::new(packed_data);
        self.push_grow(&aligned);
    }
}



#[test]
fn test_value_stack() {
    let mut stack: Stack = Stack::with_capacity(100);

    // Push multiple items onto the stack
    let value_nil = IRValue::Nil;
    let value_bool = IRValue::Bool(true);
    let value_atom = IRValue::Atom(123);
    let value_string = IRValue::String(456);
    let value_func = IRValue::Func(789);
    let value_int = IRValue::Int(42);
    let value_float = IRValue::Float(3.14);

    stack.push_value(&value_nil).unwrap();
    stack.push_value(&value_bool).unwrap();
    stack.push_value(&value_atom).unwrap();
    stack.push_value(&value_string).unwrap();
    stack.push_value(&value_func).unwrap();
    stack.push_value(&value_int).unwrap();
    stack.push_value(&value_float).unwrap();

    // Pop items one by one and check them
    let popped_value = stack.pop_value().expect("Failed to pop Value::Float");
    assert_eq!(popped_value, IRValue::Float(3.14));

    let popped_value = stack.pop_value().expect("Failed to pop Value::Int");
    assert_eq!(popped_value, IRValue::Int(42));

    let popped_value = stack.pop_value().expect("Failed to pop Value::Func");
    assert_eq!(popped_value, IRValue::Func(789));

    let popped_value = stack.pop_value().expect("Failed to pop Value::String");
    assert_eq!(popped_value, IRValue::String(456));

    let popped_value = stack.pop_value().expect("Failed to pop Value::Atom");
    assert_eq!(popped_value, IRValue::Atom(123));

    let popped_value = stack.pop_value().expect("Failed to pop Value::Bool");
    assert_eq!(popped_value, IRValue::Bool(true));

    let popped_value = stack.pop_value().expect("Failed to pop Value::Nil");
    assert_eq!(popped_value, IRValue::Nil);

    // Additional tests for specific pop methods
    stack.push_value(&value_atom).unwrap();
    stack.push_value(&value_string).unwrap();
    stack.push_value(&value_int).unwrap();

    let _popped_int = stack.pop_int().unwrap();
    let _popped_string = stack.pop_string().unwrap();
    let popped_atom = stack.pop_atom().unwrap();
    assert_eq!(popped_atom, 123);
}