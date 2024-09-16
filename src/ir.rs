#![allow(dead_code)]
use std::collections::HashMap;
use std::rc::Rc;
use codespan::Span;

pub use crate::basic_ops::{is_equal};

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum Scopble<'parent>{
    None,
    Dyn(&'parent VarScope<'parent>),
    Static(&'parent HashMap<usize,Value>),
}

#[derive(Debug,PartialEq,Clone)]
pub struct VarScope<'parent> {
	parent : Scopble<'parent>,//Option<&'parent VarScope<'parent>>,
	vars : HashMap<usize,Value>
}

impl<'parent> VarScope<'parent>  {
	pub fn new() -> Self {
		VarScope{
			parent:Scopble::None,
			vars:HashMap::new()
		}
	}
	pub fn make_subscope<'a>(&'a self) -> VarScope<'a> {
		VarScope{
			parent:Scopble::Dyn(self),
			vars:HashMap::new()
		}
	}

	pub fn add(&mut self,id : usize , val: Value) {
		self.vars.insert(id,val);
	}

	pub fn get(&self,id:usize) -> Option<&Value> {
		match self.vars.get(&id) {
			Some(v) => Some(v),
			None => match self.parent {
				Scopble::None => None,
				Scopble::Dyn(p) => p.get(id),
                Scopble::Static(p) => p.get(&id),
			}
		}
	}

    pub fn capture_entire_scope(self) -> StaticVarScope {
        let mut all_vars = HashMap::new();
        let mut current_scope = Scopble::Dyn(&self);


        while let Scopble::Dyn(scope) = current_scope {
            //respect existing values
            for (id, value) in &scope.vars {
                all_vars.entry(*id).or_insert_with(|| value.clone());
            }

            current_scope = scope.parent;
        }

        if let Scopble::Static(table) = current_scope {
            for (id, value) in table {
                all_vars.entry(*id).or_insert_with(|| value.clone());
            }
        }

        StaticVarScope { vars: all_vars }
    }
}

#[test]
fn test_scope_lifetimes(){
	let g = VarScope::new();
	let mut a = g.make_subscope();
	let _b = g.make_subscope();
	{
		let _c = a.make_subscope();
	}
	let _d = &mut a;
}


#[derive(Debug,PartialEq,Clone)]
pub struct StaticVarScope {
    vars : HashMap<usize,Value>,
}

impl StaticVarScope {
    pub fn new() -> Self {
        StaticVarScope{vars: HashMap::new()}
    }
    pub fn make_subscope<'parent>(&'parent self) -> VarScope<'parent> {
        VarScope{
            parent:Scopble::Static(&self.vars),
            vars:HashMap::new(),
        }
    }
}



#[derive(Debug, PartialEq, Clone)]
pub enum GenericRet<T> {
    //holds the return value marks what to do with the call stack
    //this is shared between match statments and functions
    //HOWEVER only matchstatments may return the Unwind varient

    Local(T),
    Unwind(T),
}

impl<T> GenericRet<T> {
    pub fn as_local(self) -> Self {
        match self {
            GenericRet::Local(v) => GenericRet::Local(v),
            GenericRet::Unwind(v) => GenericRet::Local(v),
        }
    }
    pub fn into(self) -> T {
        match self {
            GenericRet::Local(v) | GenericRet::Unwind(v) => v,
        }
    }

    pub fn map<U, F>(self, f: F) -> GenericRet<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            GenericRet::Local(v) => GenericRet::Local(f(v)),
            GenericRet::Unwind(v) => GenericRet::Unwind(f(v)),
        }
    }
}

// Define the types for ScopeRet and ValueRet using the generic enum

pub type ScopeRet = GenericRet<LazyVal>;
pub type ValueRet = GenericRet<Value>;

impl From<Value> for ValueRet {
    fn from(value: Value) -> Self {
        ValueRet::Local(value)
    }
}

impl From<ValueRet> for Value {
    fn from(ret: ValueRet) -> Self {
        match ret {
            ValueRet::Local(v) => v,
            ValueRet::Unwind(v) => v,
        }
    }
}


pub type GcPointer<T> = Rc<T>;


#[derive(Debug,PartialEq,Clone)]
pub enum LazyVal{
    //this represents a value before computation
    //however early return complicate things 
    //so we can actually return from the parent scope
    //sometimes evaluating a var causes an excepsion
    
    MakeMatchFunc(/* placeholder */),
    MakeFunc(/* placeholder */),

    Terminal(Value),
    Ref(usize),
    FuncCall(Call),
    //this is scoped for now but in some cases that makes no sense
    Match{var: Box<LazyVal>,statment: MatchStatment}
}

impl LazyVal {
    pub fn eval(&self,scope: &VarScope) -> Result<ValueRet,Error> {
        match self {
            LazyVal::Terminal(v) => Ok(v.clone().into()),
            LazyVal::Ref(id) => match scope.get(*id) {
                None => Err(Error::Missing(UndefinedName{})),
                Some(v) => Ok(v.clone().into()),
            },
            LazyVal::FuncCall(call) => call.eval(scope),
            LazyVal::Match { var, statment } => {

                match var.eval(scope)? {    
                    ValueRet::Unwind(v) => Ok(ValueRet::Unwind(v)),
                    ValueRet::Local(v) => statment.eval(v, scope),
                }
            },
            LazyVal::MakeFunc() => todo!(),
            LazyVal::MakeMatchFunc() => todo!(),

        }
    }
}

#[derive(Debug,PartialEq,Clone)]
pub enum Statment {
    Assign(usize, LazyVal),
    Call(Call),
    Match((LazyVal,MatchStatment)),
    Return(ScopeRet),
}


#[derive(Debug,PartialEq,Clone)]
pub struct Func {
    sig:FuncSig,
    closure:StaticVarScope,
    inner:Block,
}

impl Func {
    pub fn eval(&self,args: Vec<Value>) -> Result<Value,Error> {
        _ = self.sig.matches(&args)?;
        let mut scope = self.closure.make_subscope();
        for (i,a) in self.sig.arg_ids.iter().enumerate(){
            scope.add(*a,args[i].clone());
        }
        
        self.inner.eval(&scope).map(|x| x.into())
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct FuncSig{
    arg_ids: Vec<usize>, //for now its all simple
}

impl FuncSig {
   pub fn matches(&self,args: &[Value]) -> Result<(),Error> {
        if args.len() == self.arg_ids.len() {
            Ok(())
        } else {
            Err(Error::Sig(SigError{}))
        }
   } 
}

#[derive(Debug,PartialEq,Clone)]
pub enum Block{
    Simple(LazyVal),
    Code(Vec<Statment>),
}


impl Block {
    pub fn eval(&self,scope: &VarScope) -> Result<ValueRet,Error>{
        match self {
            Block::Simple(v) => v.eval(scope),
            Block::Code(c)=>Self::evaluate_code(c,scope),
        }
    }

    #[inline]
    fn evaluate_code(code:&[Statment],parent_scope: &VarScope)-> Result<ValueRet,Error>{
        let mut scope = parent_scope.make_subscope();

        for s in code.iter() {
            match s {
                Statment::Return(a) => match a{
                    GenericRet::Local(x) => {return x.eval(&scope);},
                    GenericRet::Unwind(x)=> {return x.eval(&scope);},
                },
                Statment::Assign(id,a) =>{
                    let ret = a.eval(&scope)?;
                    match ret{
                        ValueRet::Local(x) => {scope.add(*id,x);},
                        ValueRet::Unwind(_) => {return Ok(ret);}
                    }
                    
                },
                Statment::Call(v) => {
                    _ = v.eval(&scope)?;
                },
                Statment::Match((val,statment)) => {
                    let  r = val.eval(&scope)?;
                    let x = match r {
                        ValueRet::Local(x) => x,
                        ValueRet::Unwind(_) =>{return Ok(r);},
                    };
                    _ = statment.eval(x,&scope)?;
                },
            }
        }

        Ok(ValueRet::Local(Value::Nil))
    }
}



#[derive(Debug,PartialEq,Clone)]
pub enum MatchCond {
    Literal(Value),
    Any
}

impl MatchCond {
    pub fn matches(&self,v: &Value) -> bool{
        match self {
            MatchCond::Any => true,
            MatchCond::Literal(x) => is_equal(v,x),
        }
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct MatchStatment {
    arms: Vec<MatchCond>,
    vals: Vec<Block>,
    debug_span: Span,
}

impl MatchStatment {
    pub fn eval(&self, x:Value ,scope: &VarScope) -> Result<ValueRet,Error> {
        for (i,a) in self.arms.iter().enumerate() {
            if a.matches(&x) {
                return self.vals[i].eval(scope);
            }
        }
        return Err(Error::Match(MatchError{span:self.debug_span}));
    }
}

#[derive(Debug,PartialEq,Clone)]
pub enum FunctionHandle{
    FFI(fn(Vec<Value>)->Result<Value,Error>),
	StaticDef(&'static Func),
    Lambda(GcPointer<Func>),
    //we need a better premise for matchlamdas or just make them Funcs
    // MatchLambda(GcPointer<MatchStatment>),
    
}

impl FunctionHandle{
    pub fn eval(self,args: Vec<Value>) -> Result<Value,Error> {
        match self {
            FunctionHandle::FFI(f) => f(args),
            FunctionHandle::StaticDef(f) => f.eval(args),
            FunctionHandle::Lambda(f) => f.eval(args),

            // FunctionHandle::MatchLambda(l) => {
            //     if args.len()!=1 {
            //         return Err(Error::Sig(SigError{}));
            //     }
            //     let ret = l.eval(args[0].clone(),&VarScope::new())?;
            //     Ok(ret.into())
            // },
        }
        
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct Call{
    //handle:Box<FunctionHandle>,
    called: Box<LazyVal>,
    args: Vec<LazyVal>
}

impl Call {
    pub fn eval(&self,scope: &VarScope) -> Result<ValueRet,Error> {
        let handle = match self.called.eval(scope)? {
            ValueRet::Unwind(v) => {return Ok(ValueRet::Unwind(v));}
            ValueRet::Local(v) => match v {
                Value::Func(f) => f,
                _ => {return Err(Error::NoneCallble(NoneCallble{}));}
            }
        };

        let mut arg_values = Vec::with_capacity(self.args.len());
        for a in self.args.iter() {
            match a.eval(scope) {
                Err(e) => {return Err(e);},
                Ok(x) => {match x {
                    ValueRet::Local(v) => {arg_values.push(v);},
                    ValueRet::Unwind(_) => {return Ok(x);},
                }}
            };
        }
        handle.eval(arg_values).map(|v| v.into())
    }
}


#[derive(Debug,PartialEq,Clone)]
pub enum Value {
	Nil,
	Bool(bool),
	Atom(usize),
	String(GcPointer<String>),
	Int(i64),
	Float(f64),
	Func(FunctionHandle),
}

#[derive(Debug,PartialEq)]
pub enum Error {
    Match(MatchError),
    Sig(SigError),
    Missing(UndefinedName),
    NoneCallble(NoneCallble)
    //UndocumentedError,
}

#[derive(Debug,PartialEq)]
pub struct MatchError {
    span: Span
}


#[derive(Debug,PartialEq)]
pub struct SigError {
	//placeholder
}

#[derive(Debug,PartialEq)]
pub struct NoneCallble {
    //placeholder
}


#[derive(Debug,PartialEq)]
pub struct UndefinedName {
    //placeholder
}

#[cfg(test)]
use std::cell::RefCell;

#[cfg(test)]
use crate::ast::StringTable;

#[test]
fn test_system_ffi_mock() {
    //define FFI functions for this test with loging
    //with this testing buffer 
    thread_local! {
        static LOG_PRINT: RefCell<Vec<Vec<Value>>> = RefCell::new(Vec::new());
        static LOG_SYSTEM: RefCell<Vec<Vec<Value>>> = RefCell::new(Vec::new());
    }


    fn ffi_println(args: Vec<Value>) -> Result<Value, Error> {
        LOG_PRINT.with(|log| {
            log.borrow_mut().push(args.clone());
        });
        Ok(Value::Nil)
    }

    fn ffi_system(args: Vec<Value>) -> Result<Value, Error> {
        LOG_SYSTEM.with(|log| {
            log.borrow_mut().push(args.clone());
        });

        Ok(Value::Func(FunctionHandle::FFI(ffi_println)))
    }

    //initilize scope

    let mut string_table = StringTable::new();
    let system_name = string_table.get_id("system");
    let println_name = string_table.get_id(":println");

    let mut scope = VarScope::new();
    scope.add(system_name, Value::Func(FunctionHandle::FFI(ffi_system)));

    // Inner call for accessing the `system` variable from the scope
    let system_call = Call {
        called: Box::new(LazyVal::Ref(system_name)),
        args: vec![LazyVal::Terminal(Value::Atom(println_name))],
    };

    // Outer call for `system(:println)("hello world")`
    let outer_call = Call {
        called: Box::new(LazyVal::FuncCall(system_call)),
        args: vec![LazyVal::Terminal(Value::String(GcPointer::new("hello world".to_string())))],
    };

    //asserts

    let ans = outer_call.eval(&scope).unwrap();
    assert_eq!(ans, ValueRet::Local(Value::Nil));

    LOG_SYSTEM.with(|log| {
        assert_eq!(log.borrow().len(), 1);
        assert_eq!(log.borrow()[0], vec![Value::Atom(println_name)]);
    });

    LOG_PRINT.with(|log| {
        assert_eq!(log.borrow().len(), 1);
        assert_eq!(log.borrow()[0], vec![Value::String(GcPointer::new("hello world".to_string()))]);
    });
}

#[test]
fn test_varscope_add_and_get() {
    let mut scope = VarScope::new();
    let id = 1;
    let val = Value::Int(42);
    
    scope.add(id, val.clone());
    
    // Check if the value can be retrieved
    assert_eq!(scope.get(id), Some(&val));
    
    // Check a non-existent value
    assert_eq!(scope.get(2), None);
}

#[test]
fn test_varscope_nested_scopes() {
    let mut global_scope = VarScope::new();
    let id = 1;
    let val = Value::Int(42);
    
    global_scope.add(id, val.clone());
    
    // Create a subscope and check if it can access the parent value
    let subscope = global_scope.make_subscope();
    assert_eq!(subscope.get(id), Some(&val));
    
    // Add a new value in the subscope and check it
    let sub_id = 2;
    let sub_val = Value::Bool(true);
    let mut mutable_subscope = subscope.make_subscope();
    mutable_subscope.add(sub_id, sub_val.clone());
    
    assert_eq!(mutable_subscope.get(sub_id), Some(&sub_val));
    assert_eq!(mutable_subscope.get(id), Some(&val)); // Should still be able to access parent's value
}

#[test]
fn test_function_handle_eval() {
    let func = Func {
        sig: FuncSig { arg_ids: vec![1, 2] },
        inner: Block::Simple(LazyVal::Terminal(Value::Int(42))),
        closure: StaticVarScope::new()
    };

    let handle = FunctionHandle::Lambda(Rc::new(func));
    
    // Test valid evaluation with matching arguments
    let result = handle.clone().eval(vec![Value::Int(1), Value::Int(2)]).unwrap();
    assert_eq!(result, Value::Int(42));

    // Test invalid evaluation with incorrect number of arguments
    let err = handle.eval(vec![Value::Int(1)]).unwrap_err();
    assert_eq!(err, Error::Sig(SigError {}));
}

#[test]
fn test_match_statement() {
    let match_stmt = MatchStatment {
        arms: vec![
            MatchCond::Literal(Value::Int(1)),
            MatchCond::Literal(Value::Int(2)),
            MatchCond::Any
        ],
        vals: vec![
            Block::Simple(LazyVal::Terminal(Value::String(Rc::new("One".to_string())))),
            Block::Simple(LazyVal::Terminal(Value::String(Rc::new("Two".to_string())))),
            Block::Simple(LazyVal::Terminal(Value::String(Rc::new("Default".to_string()))))
        ],
        debug_span: Span::new(0, 0),
    };
    
    let scope = VarScope::new();
    
    // Test matching on a specific value
    let result_one = match_stmt.eval(Value::Int(1), &scope).unwrap();
    assert_eq!(result_one, ValueRet::Local(Value::String(Rc::new("One".to_string()))));

    let result_two = match_stmt.eval(Value::Int(2), &scope).unwrap();
    assert_eq!(result_two, ValueRet::Local(Value::String(Rc::new("Two".to_string()))));

    // Test matching on a default case
    let result_default = match_stmt.eval(Value::Int(3), &scope).unwrap();
    assert_eq!(result_default, ValueRet::Local(Value::String(Rc::new("Default".to_string()))));
}

#[test]
fn test_lazyval_func_call() {
    let func = Func {
        sig: FuncSig { arg_ids: vec![1] },
        inner: Block::Simple(LazyVal::Terminal(Value::Int(42))),
        closure: StaticVarScope::new()
    };
    let handle = Value::Func(FunctionHandle::Lambda(Rc::new(func)));
    let scope = VarScope::new();

    // Create a function call LazyVal
    let call = Call {
        called: Box::new(LazyVal::Terminal(handle.clone())),
        args: vec![LazyVal::Terminal(Value::Int(5))],
    };

    let result = LazyVal::FuncCall(call).eval(&scope).unwrap();
    assert_eq!(result, ValueRet::Local(Value::Int(42)));
}

#[test]
fn test_match_statement_with_ref_and_func_call() {
    // Create a VarScope and add a referenced value
    let mut scope = VarScope::new();
    let ref_id = 10;
    let ref_value = Value::String(Rc::new("Referenced".to_string()));
    scope.add(ref_id, ref_value.clone());

    // Define a simple function that returns a specific value
    let func = Func {
        sig: FuncSig { arg_ids: vec![23] },
        inner: Block::Simple(LazyVal::Terminal(Value::String(Rc::new("FunctionCall".to_string())))),
        closure: StaticVarScope::new()
    };
    let handle = Value::Func(FunctionHandle::Lambda(Rc::new(func)));

    // Create the match statement
    let match_stmt = MatchStatment {
        arms: vec![
            MatchCond::Literal(Value::Int(1)),
            MatchCond::Literal(Value::Int(2)),
            MatchCond::Any
        ],
        vals: vec![
            Block::Simple(LazyVal::Terminal(Value::String(Rc::new("One".to_string())))),    // Terminal value
            Block::Simple(LazyVal::Ref(ref_id)),                                             // Reference to a value in scope
            Block::Simple(LazyVal::FuncCall(Call {                                          // Function call returning "FunctionCall"
                called: Box::new(LazyVal::Terminal(handle.clone())),                        // Call the function
                args: vec![LazyVal::Terminal(Value::Float(6.9))],                               // Pass an argument (though it's unused in this example)
            }))
        ],
        debug_span: Span::new(0, 0),
    };

    // Test matching on a specific value (1)
    let result_one = match_stmt.eval(Value::Int(1), &scope).unwrap();
    assert_eq!(result_one, ValueRet::Local(Value::String(Rc::new("One".to_string()))));

    // Test matching on a specific value (2) which is a Ref
    let result_two = match_stmt.eval(Value::Int(2), &scope).unwrap();
    assert_eq!(result_two, ValueRet::Local(ref_value));  // Should match the referenced value

    // Test matching on a default case, which is a function call
    let result_default = match_stmt.eval(Value::Int(3), &scope).unwrap();
    assert_eq!(result_default, ValueRet::Local(Value::String(Rc::new("FunctionCall".to_string()))));
}

#[test]
fn test_closure_variable_isolation() {
    // Create a global scope and add a variable
    let mut global_scope = VarScope::new();
    let global_var_id = 1;
    let global_value = Value::Int(100);
    global_scope.add(global_var_id, global_value.clone());

    // Create a function that will modify its own scope but should not modify the outer/global scope
    let func = Func {
        sig: FuncSig { arg_ids: vec![2] },
        inner: Block::Code(vec![
            // Inside the function, we assign a new value to the same variable ID (1)
            Statment::Assign(global_var_id, LazyVal::Terminal(Value::Int(200))),
        ]),
        closure: global_scope.clone().capture_entire_scope(), // Function has its own isolated closure
    };


    // Function handle to represent our closure
    let handle = FunctionHandle::Lambda(Rc::new(func));

    // Call the function, passing an argument (though it's not used)
    handle.clone().eval(vec![Value::Int(5)]).unwrap();

    // The global scope should still have the original value, as the closure should not modify it
    assert_eq!(global_scope.get(global_var_id), Some(&global_value));

    // Modify the global scope directly to verify that inner scopes aren't affecting the global scope
    let modified_global_value = Value::Int(300);
    global_scope.add(global_var_id, modified_global_value.clone());

    // Now call the function again
    handle.eval(vec![Value::Int(5)]).unwrap();

    // Verify the function's internal scope is isolated and does not affect the outer scope
    assert_eq!(global_scope.get(global_var_id), Some(&modified_global_value));
}

#[test]
fn test_closure_does_not_leak_into_global_scope() {
    // Create the global scope
    let mut global_scope = VarScope::new();
    let global_var_id = 1;
    let global_value = Value::Int(100);
    global_scope.add(global_var_id, global_value.clone());

    // Create a function that modifies its own local scope and does not leak variables
    let func = Func {
        sig: FuncSig { arg_ids: vec![2] },
        inner: Block::Code(vec![
            // Assign a value to a new variable ID (2) that should exist only within the function
            Statment::Assign(2, LazyVal::Terminal(Value::Int(500))),
        ]),
        closure: global_scope.clone().capture_entire_scope(), // Function has its own closure
    };

    // Create a handle for the function
    let handle = FunctionHandle::Lambda(Rc::new(func));

    // Call the function, passing an argument (though it's not used)
    handle.eval(vec![Value::Int(5)]).unwrap();

    // Verify that the new variable (ID 2) does not exist in the global scope
    assert_eq!(global_scope.get(2), None);

    // Ensure that the global variable (ID 1) has not been modified by the closure
    assert_eq!(global_scope.get(global_var_id), Some(&global_value));
}
