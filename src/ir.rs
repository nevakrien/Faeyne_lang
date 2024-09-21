#![allow(dead_code)]
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use std::rc::Rc;
use codespan::Span;

pub use crate::basic_ops::{is_equal};
use crate::reporting::*;

#[derive(Debug,PartialEq,Clone)]
#[derive(Default)]
pub struct GlobalScope {
    vars : HashMap<usize,(FuncSig,Block)>,
}



impl GlobalScope {
    pub fn get(&'static self,id:usize) -> Option<Value> {
        let (sig,inner) = self.vars.get(&id)?;
        Some(Value::Func(FunctionHandle::StaticDef(Box::new(
            GlobalFunc{
                sig:sig.clone(),
                inner:inner.clone(),
                global : self
            }
        ))))
    }

    pub fn add(&mut self, id: usize, block: Block, sig: FuncSig) -> Result<(), ErrList> {
        if let std::collections::hash_map::Entry::Vacant(e) = self.vars.entry(id) {
            e.insert((sig, block));
            Ok(())
        } else {
            // TODO: Handle this case with more complex behavior when adding multiple catching patterns
            // For now, we mark this as a TODO to indicate the behavior needs more consideration.
            Err(Error::UnreachableCase(UnreachableCase{
                name:id,
                sig,
            }).to_list())
        }
    }


    pub fn make_subscope(&'static self) -> VarScope<'_> {
        VarScope{
            parent:Scopble::Global(self),
            vars:HashMap::new()
        }
    }
}

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum Scopble<'a>{
    Global(&'static GlobalScope),
    SubScope(&'a VarScope<'a>),
    Static(&'a StaticVarScope),
}

impl<'a> Scopble<'a>{
    pub fn get(&self,id:usize) -> Option<Value> {
        match self{
            Scopble::Global(s) => s.get(id),
            Scopble::SubScope(s) => s.get(id),
            Scopble::Static(s) => s.get(id),
        }
    }

    pub fn get_root(&'a self) -> Scopble<'a> {
        match self {
            Scopble::SubScope(s) => s.parent.get_root(),
            Scopble::Static(s) => Scopble::Static(s),
            Scopble::Global(s) => Scopble::Global(s),
        }
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct VarScope<'parent> {
	parent : Scopble<'parent>,
	vars : HashMap<usize,Value>
}


impl<'parent> VarScope<'parent>  {
    pub fn new(parent: Scopble<'parent>) -> Self {
		VarScope{
			parent,
			vars:HashMap::new()
		}
	}
	pub fn make_subscope(&self) -> VarScope<'_> {
		VarScope{
			parent:Scopble::SubScope(self),
			vars:HashMap::new()
		}
	}

	pub fn add(&mut self,id : usize , val: Value) {
		self.vars.insert(id,val);
	}

	pub fn get(&self,id:usize) -> Option<Value> {
		match self.vars.get(&id) {
			Some(v) => Some(v.clone()),
			None => self.parent.get(id),
		}
	}

    pub fn capture_entire_scope(&self) -> StaticVarScope {
        let mut all_vars = HashMap::new();
        let mut current_scope = Scopble::SubScope(self);


        while let Scopble::SubScope(scope) = current_scope {
            //respect existing values
            for (id, value) in &scope.vars {
                all_vars.entry(*id).or_insert_with(|| value.clone());
            }

            current_scope = scope.parent;
        }

        if let Scopble::Static(scope) = current_scope {
            for (id, value) in &scope.vars {
                all_vars.entry(*id).or_insert_with(|| value.clone());
            }
        }

        StaticVarScope { vars: all_vars }
    }
}

#[test]
fn test_scope_lifetimes(){
    let r = StaticVarScope::new();
	let g = VarScope::new(Scopble::Static(&r));
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

impl Default for StaticVarScope {
    fn default() -> Self {
        Self::new()
    }
}

impl StaticVarScope {
    pub fn new() -> Self {
        StaticVarScope{vars: HashMap::new()}
    }

    pub fn get(&self,id:usize) -> Option<Value> {
        self.vars.get(&id).cloned()
    }

    pub fn maybe_add(&mut self,id : usize ,outer_scope: &VarScope<'_>) -> Result<(),ErrList> {
        match self.vars.entry(id){
            Entry::Occupied(_) => Ok(()), 
            Entry::Vacant(spot) => {
                match outer_scope.get(id) {
                    None => Err(Error::Missing(UndefinedName{}).to_list()),
                    Some(v) => {spot.insert(v.clone()); Ok(())}
                }
            }
        }
    }


    pub fn make_subscope(&self) -> VarScope<'_> {
        VarScope{
            parent:Scopble::Static(self),
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

impl From<ScopeRet> for LazyVal {
    fn from(ret: ScopeRet) -> Self {
        match ret {
            ScopeRet::Local(v) => v,
            ScopeRet::Unwind(v) => v,
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
    
    MakeMatchFunc(LazyMatch),
    MakeFunc(LazyFunc),

    Terminal(Value),
    Ref(usize),
    FuncCall(Call),
    //this is scoped for now but in some cases that makes no sense
    Match{var: Box<LazyVal>,statment: MatchStatment}
}

impl LazyVal {
    pub fn eval(&self,scope: &VarScope) -> Result<ValueRet,ErrList> {
        match self {
            LazyVal::Terminal(v) => Ok(v.clone().into()),
            LazyVal::Ref(id) => match scope.get(*id) {
                None => Err(Error::Missing(UndefinedName{}).to_list()),
                Some(v) => Ok(v.clone().into()),
            },
            LazyVal::FuncCall(call) => call.eval(scope),
            LazyVal::Match { var, statment } => {

                match var.eval(scope)? {    
                    ValueRet::Unwind(v) => Ok(ValueRet::Unwind(v)),
                    ValueRet::Local(v) => statment.eval(v, scope),
                }
            },
            LazyVal::MakeFunc(lf) => lf.eval(scope).map(|f| 
                ValueRet::Local(
                    Value::Func(
                        FunctionHandle::Lambda(
                            GcPointer::new(f)
                        ) 
                    )
                )
            ),
            LazyVal::MakeMatchFunc(x) => x.eval(scope).map(|f| 
                ValueRet::Local(
                    Value::Func(
                        FunctionHandle::Lambda(
                            GcPointer::new(f)
                        ) 
                    )
                )
            ),

        }
    }


    pub fn add_to_closure(&self,scope: &VarScope<'_>,closure : &mut StaticVarScope) -> Result<(),ErrList> {
        match self {
            LazyVal::Terminal(_) => Ok(()),
            LazyVal::Ref(id) => closure.maybe_add(*id,scope),
            LazyVal::Match{ var, statment }=> {
                append_err_list(
                    var.add_to_closure(scope,closure),
                    statment.add_to_closure(scope,closure)
                )
            },

            LazyVal::FuncCall(call) => call.add_to_closure(scope,closure),
            LazyVal::MakeFunc(lf) => lf.add_to_closure(scope,closure),
            LazyVal::MakeMatchFunc(x) => x.add_to_closure(scope,closure),
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

impl Statment{
    pub fn add_to_closure(&self,scope: &VarScope<'_>,closure : &mut StaticVarScope) -> Result<(),ErrList>{
        match self{
            Statment::Assign(_,x) => x.add_to_closure(scope,closure),
            Statment::Call(x) => x.add_to_closure(scope,closure),
            Statment::Return(r) => {
                let x : LazyVal = r.clone().into(); 
                x.add_to_closure(scope,closure)
            },
            Statment::Match((val,s)) => append_err_list(
                val.add_to_closure(scope,closure),
                s.add_to_closure(scope,closure)
            ),

        }
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct LazyFunc{
    sig:FuncSig,
    inner:Block,
}

impl LazyFunc {
    pub fn new(sig : FuncSig,inner : Block) -> Self {
       LazyFunc{
            sig,inner
       }
    }
    pub fn eval(&self,scope: &VarScope) -> Result<Func,ErrList> {
        let mut closure = StaticVarScope::new();
    
        self.inner.add_to_closure(scope,&mut closure)?;        
        Ok(Func {
            sig:self.sig.clone(),
            inner: self.inner.clone(),
            closure
        })
    }

    pub fn add_to_closure(&self,scope: &VarScope<'_>,closure : &mut StaticVarScope) -> Result<(),ErrList> {
        self.inner.add_to_closure(scope,closure)
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct LazyMatch{
    inner:MatchStatment,
}

impl LazyMatch {
    pub fn new(inner : MatchStatment) -> Self {
       LazyMatch{
            inner
       }
    }
    pub fn eval(&self, scope: &VarScope) -> Result<Func, ErrList> {
        let mut closure = StaticVarScope::new();

        self.inner.add_to_closure(scope, &mut closure)?;

        //we are using some cursed ideas basically the 0 id will never be a varible name
        // thus using it as an argument is totally safe

        let statment = Statment::Return(
            ScopeRet::Local(
                LazyVal::Match {
                    var: Box::new(LazyVal::Ref(0)),
                    statment: self.inner.clone()    
                }
            )
        );

        Ok(Func {
            sig: FuncSig { arg_ids: vec![0] },
            inner: Block { code: vec![statment] }, 
            closure,
        })
    }


    pub fn add_to_closure(&self,scope: &VarScope<'_>,closure : &mut StaticVarScope) -> Result<(),ErrList> {
        self.inner.add_to_closure(scope,closure)
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct GlobalFunc {
    sig:FuncSig,
    global: &'static GlobalScope,
    inner:Block,
}

impl GlobalFunc {
    pub fn eval(&self,args: Vec<Value>) -> Result<Value,ErrList> {
        self.sig.matches(&args)?;
        let mut scope = self.global.make_subscope();
        for (i,a) in self.sig.arg_ids.iter().enumerate(){
            scope.add(*a,args[i].clone());
        }
        
        self.inner.eval(&scope).map(|x| x.into())
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct Func {
    sig:FuncSig,
    closure:StaticVarScope,
    inner:Block,
}

impl Func {
    pub fn eval(&self,args: Vec<Value>) -> Result<Value,ErrList> {
        self.sig.matches(&args)?;
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
   pub fn matches(&self,args: &[Value]) -> Result<(),ErrList> {
        if args.len() == self.arg_ids.len() {
            Ok(())
        } else {
            Err(Error::Sig(SigError{}).to_list())
        }
   }
}

#[derive(Debug,PartialEq,Clone)]
pub struct Block{
    code: Vec<Statment>,
}


impl Block {
    pub fn new(code : Vec<Statment>) -> Self {
        Block{code}
    }

    pub fn new_simple(val :LazyVal) -> Self {
        Block {
            code: vec![Statment::Return(GenericRet::Local(val))]
        }
    }

    pub fn eval(&self,parent_scope: &VarScope)-> Result<ValueRet,ErrList>{
        let mut scope = parent_scope.make_subscope();

        for s in self.code.iter() {
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

    pub fn add_to_closure(&self,scope: &VarScope<'_>,closure : &mut StaticVarScope) -> Result<(),ErrList>{
        let mut ans = Ok(());
        for s in self.code.iter() {
            ans = append_err_list(ans,
                s.add_to_closure(scope,closure)
            );
        }
        ans
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
    pub fn add_to_closure(&self,_scope: &VarScope<'_>,_closure : &mut StaticVarScope) -> Result<(),ErrList>{
        Ok(())
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct MatchStatment {
    arms: Vec<MatchCond>,
    vals: Vec<Block>,
    debug_span: Span,
}

impl MatchStatment {
    pub fn new (arms: Vec<MatchCond>,vals: Vec<Block>,debug_span: Span,) -> Self {
        MatchStatment{arms,vals,debug_span}
    }
    pub fn eval(&self, x:Value ,scope: &VarScope) -> Result<ValueRet,ErrList> {
        for (i,a) in self.arms.iter().enumerate() {
            if a.matches(&x) {
                return self.vals[i].eval(scope);
            }
        }
        Err(Error::Match(MatchError{span:self.debug_span}).to_list())
    }

    pub fn add_to_closure(&self,scope: &VarScope<'_>,closure : &mut StaticVarScope) -> Result<(),ErrList> {
        let mut ans = Ok(());
        for a in self.arms.iter() {
            ans=append_err_list(ans,a.add_to_closure(scope,closure));
        }
        for v in self.vals.iter() {
            ans=append_err_list(ans,v.add_to_closure(scope,closure));
        }
        ans
    }
}

#[derive(Debug,PartialEq,Clone)]
pub enum FunctionHandle{
    FFI(fn(Vec<Value>)->Result<Value,ErrList>),
    StaticDef(Box<GlobalFunc>),
    Lambda(GcPointer<Func>),
    //we need a better premise for matchlamdas or just make them Funcs
    // MatchLambda(GcPointer<MatchStatment>),
    
}

impl FunctionHandle{
    pub fn eval(self,args: Vec<Value>) -> Result<Value,ErrList> {
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
    pub called: Box<LazyVal>,
    pub args: Vec<LazyVal>,
    pub debug_span: Span
}

impl Call {
    pub fn eval(&self,scope: &VarScope) -> Result<ValueRet,ErrList> {
        let handle = match self.called.eval(scope)? {
            ValueRet::Unwind(v) => {return Ok(ValueRet::Unwind(v));}
            ValueRet::Local(v) => match v {
                Value::Func(f) => f,
                _ => {return Err(Error::NoneCallble(NoneCallble{}).to_list());}
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
    pub fn add_to_closure(&self,scope: &VarScope<'_>,closure : &mut StaticVarScope) -> Result<(),ErrList> {
        let mut ans = self.called.add_to_closure(scope,closure);
        for a in self.args.iter() {
            ans = append_err_list(ans,
                a.add_to_closure(scope,closure)
            );
        }
        ans
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



#[cfg(test)]
use std::cell::RefCell;

#[cfg(test)]
use crate::ast::StringTable;

#[test]
fn test_system_ffi_mock() {
    //define FFI functions for this test with loging
    //with this testing buffer 
    thread_local! {
        static LOG_PRINT: RefCell<Vec<Vec<Value>>> = const { RefCell::new(Vec::new()) };
        static LOG_SYSTEM: RefCell<Vec<Vec<Value>>> = const { RefCell::new(Vec::new()) };
    }


    fn ffi_println(args: Vec<Value>) -> Result<Value, ErrList> {
        LOG_PRINT.with(|log| {
            log.borrow_mut().push(args.clone());
        });
        Ok(Value::Nil)
    }

    fn ffi_system(args: Vec<Value>) -> Result<Value, ErrList    > {
        LOG_SYSTEM.with(|log| {
            log.borrow_mut().push(args.clone());
        });

        Ok(Value::Func(FunctionHandle::FFI(ffi_println)))
    }

    //initilize scope

    let mut string_table = StringTable::new();
    let system_name = string_table.get_id("system");
    let println_name = string_table.get_id(":println");

    let root = Box::new(GlobalScope::default());
    let r = Scopble::Global(Box::leak(root));

    let mut scope = VarScope::new(r);
    scope.add(system_name, Value::Func(FunctionHandle::FFI(ffi_system)));

    // Inner call for accessing the `system` variable from the scope
    let system_call = Call {
        called: Box::new(LazyVal::Ref(system_name)),
        args: vec![LazyVal::Terminal(Value::Atom(println_name))],
        debug_span : Span::new(0,1),
    };

    // Outer call for `system(:println)("hello world")`
    let outer_call = Call {
        called: Box::new(LazyVal::FuncCall(system_call)),
        args: vec![LazyVal::Terminal(Value::String(GcPointer::new("hello world".to_string())))],
        debug_span : Span::new(0,1),
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
    let global = Box::new(GlobalScope::default()); // Create the global scope using Default
    let global_ref = Box::leak(global);            // Create a static reference
    let r = Scopble::Global(global_ref);           // Use the global scope

    let mut scope = VarScope::new(r);
    let id = 1;
    let val = Value::Int(42);
    scope.add(id, val.clone());
    
    // Check if the value can be retrieved
    assert_eq!(scope.get(id), Some(val));
    
    // Check a non-existent value
    assert_eq!(scope.get(2), None);
}


#[test]
fn test_varscope_nested_scopes() {
    let root = Box::new(GlobalScope::default());
    let r = Scopble::Global(Box::leak(root));

    let mut global_scope = VarScope::new(r);
    let id = 1;
    let val = Value::Int(42);
    
    global_scope.add(id, val.clone());
    
    // Create a subscope and check if it can access the parent value
    let subscope = global_scope.make_subscope();
    assert_eq!(subscope.get(id), Some(val.clone()));
    
    // Add a new value in the subscope and check it
    let sub_id = 2;
    let sub_val = Value::Bool(true);
    let mut mutable_subscope = subscope.make_subscope();
    mutable_subscope.add(sub_id, sub_val.clone());
    
    assert_eq!(mutable_subscope.get(sub_id), Some(sub_val));
    assert_eq!(mutable_subscope.get(id), Some(val.clone())); // Should still be able to access parent's value
}

#[test]
fn test_function_handle_eval() {

    let func = Func {
        sig: FuncSig { arg_ids: vec![1, 2] },
        inner: Block::new_simple(LazyVal::Terminal(Value::Int(42))),
        closure: StaticVarScope::new()
    };

    let handle = FunctionHandle::Lambda(Rc::new(func));
    
    // Test valid evaluation with matching arguments
    let result = handle.clone().eval(vec![Value::Int(1), Value::Int(2)]).unwrap();
    assert_eq!(result, Value::Int(42));

    // Test invalid evaluation with incorrect number of arguments
    let err = handle.eval(vec![Value::Int(1)]).unwrap_err();
    assert_eq!(err, Error::Sig(SigError {}).to_list());
}

#[test]
fn test_match_statement() {
    let root = Box::new(GlobalScope::default());
    let r = Scopble::Global(Box::leak(root));

    let match_stmt = MatchStatment {
        arms: vec![
            MatchCond::Literal(Value::Int(1)),
            MatchCond::Literal(Value::Int(2)),
            MatchCond::Any
        ],
        vals: vec![
            Block::new_simple(LazyVal::Terminal(Value::String(Rc::new("One".to_string())))),
            Block::new_simple(LazyVal::Terminal(Value::String(Rc::new("Two".to_string())))),
            Block::new_simple(LazyVal::Terminal(Value::String(Rc::new("Default".to_string()))))
        ],
        debug_span: Span::new(0, 0),
    };
    
    let scope = VarScope::new(r);
    
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
    let root = Box::new(GlobalScope::default());
    let r = Scopble::Global(Box::leak(root));

    let func = Func {
        sig: FuncSig { arg_ids: vec![1] },
        inner: Block::new_simple(LazyVal::Terminal(Value::Int(42))),
        closure: StaticVarScope::new()
    };
    let handle = Value::Func(FunctionHandle::Lambda(Rc::new(func)));
    let scope = VarScope::new(r);

    // Create a function call LazyVal
    let call = Call {
        called: Box::new(LazyVal::Terminal(handle.clone())),
        args: vec![LazyVal::Terminal(Value::Int(5))],
        debug_span : Span::new(0,1),
    };

    let result = LazyVal::FuncCall(call).eval(&scope).unwrap();
    assert_eq!(result, ValueRet::Local(Value::Int(42)));
}

#[test]
fn test_match_statement_with_ref_and_func_call() {
    let root = Box::new(GlobalScope::default());
    let r = Scopble::Global(Box::leak(root));

    // Create a VarScope and add a referenced value
    let mut scope = VarScope::new(r);
    let ref_id = 10;
    let ref_value = Value::String(Rc::new("Referenced".to_string()));
    scope.add(ref_id, ref_value.clone());

    // Define a simple function that returns a specific value
    let func = Func {
        sig: FuncSig { arg_ids: vec![23] },
        inner: Block::new_simple(LazyVal::Terminal(Value::String(Rc::new("FunctionCall".to_string())))),
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
            Block::new_simple(LazyVal::Terminal(Value::String(Rc::new("One".to_string())))),    // Terminal value
            Block::new_simple(LazyVal::Ref(ref_id)),                                             // Reference to a value in scope
            Block::new_simple(LazyVal::FuncCall(Call {                                          // Function call returning "FunctionCall"
                called: Box::new(LazyVal::Terminal(handle.clone())),                        // Call the function
                args: vec![LazyVal::Terminal(Value::Float(6.9))],
                debug_span : Span::new(0,1),
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
    let root = Box::new(GlobalScope::default());
    let r = Scopble::Global(Box::leak(root));

    // Create a global scope and add a variable
    let mut global_scope = VarScope::new(r);
    let global_var_id = 1;
    let global_value = Value::Int(100);
    global_scope.add(global_var_id, global_value.clone());

    // Create a LazyFunc that modifies its own scope but should not modify the outer/global scope
    let lazy_func = LazyFunc {
        sig: FuncSig { arg_ids: vec![2] }, // Function signature with one argument
        inner: Block::new(vec![
            // Inside the function, we assign a new value to the same variable ID (1)
            Statment::Assign(global_var_id, LazyVal::Terminal(Value::Int(200))),
        ]),
    };

    // Evaluate the function and create a closure
    let func = lazy_func.eval(&global_scope).unwrap();
    let handle = FunctionHandle::Lambda(Rc::new(func));

    // Call the function, passing an argument (though it's not used)
    handle.clone().eval(vec![Value::Int(5)]).unwrap();

    // The global scope should still have the original value, as the closure should not modify it
    assert_eq!(global_scope.get(global_var_id), Some(global_value));

    // Modify the global scope directly to verify that inner scopes aren't affecting the global scope
    let modified_global_value = Value::Int(300);
    global_scope.add(global_var_id, modified_global_value.clone());

    // Now call the function again
    handle.eval(vec![Value::Int(5)]).unwrap();

    // Verify the function's internal scope is isolated and does not affect the outer scope
    assert_eq!(global_scope.get(global_var_id), Some(modified_global_value));
}
#[test]
fn test_closure_does_not_leak_into_global_scope() {
    let root = Box::new(GlobalScope::default());
    let r = Scopble::Global(Box::leak(root));

    // Create the global scope
    let mut global_scope = VarScope::new(r);
    let global_var_id = 1;
    let global_value = Value::Int(100);
    global_scope.add(global_var_id, global_value.clone());

    // Create a LazyFunc that modifies its own local scope and does not leak variables
    let lazy_func = LazyFunc {
        sig: FuncSig { arg_ids: vec![2] }, // Function signature with one argument
        inner: Block::new(vec![
            // Assign a value to a new variable ID (2) that should exist only within the function
            Statment::Assign(2, LazyVal::Terminal(Value::Int(500))),
        ]),
    };

    // Evaluate the function and create a closure
    let func = lazy_func.eval(&global_scope).unwrap();
    let handle = FunctionHandle::Lambda(Rc::new(func));

    // Call the function, passing an argument (though it's not used)
    handle.eval(vec![Value::Int(5)]).unwrap();

    // Verify that the new variable (ID 2) does not exist in the global scope
    assert_eq!(global_scope.get(2), None);

    // Ensure that the global variable (ID 1) has not been modified by the closure
    assert_eq!(global_scope.get(global_var_id), Some(global_value));
}



#[test]
fn test_closure_captures_variable_correctly() {
    let root = Box::new(GlobalScope::default());
    let r = Scopble::Global(Box::leak(root));

    // Step 1: Create the global scope and add a variable
    let mut global_scope = VarScope::new(r);
    let var_id = 6;
    let initial_value = Value::Int(42);  // Initial value for the variable at ID 6
    global_scope.add(var_id, initial_value.clone());

    // Step 2: Create a LazyFunc (closure) that captures the variable at ID 6 and returns its value
    let lazy_func = LazyFunc {
        sig: FuncSig { arg_ids: vec![] }, // No arguments needed
        inner: Block::new(vec![
            Statment::Return(ScopeRet::Local(LazyVal::Ref(var_id))), // Return the captured value
        ]),
    };

    // Evaluate the function to create the closure
    let func = lazy_func.eval(&global_scope).unwrap();
    let handle = FunctionHandle::Lambda(Rc::new(func));

    // Step 3: Change the value of the variable in the global scope
    let new_value = Value::Int(100);  // New value for the variable at ID 6
    global_scope.add(var_id, new_value.clone());

    // Step 4: Call the closure and check that it returns the old (captured) value, not the new one
    let result = handle.eval(vec![]).unwrap();
    
    // The closure should return the old value (42) as it captured the original value
    assert_eq!(result, initial_value);

    // Confirm that the global scope has the new value (100)
    assert_eq!(global_scope.get(var_id), Some(new_value));
}

#[test]
fn test_match_fn_captures() {
    let root = Box::new(GlobalScope::default());
    let r = Scopble::Global(Box::leak(root));

    // Step 1: Create the global scope and add a variable to be captured
    let mut global_scope = VarScope::new(r);
    let capture_id = 5; // Using realistic return ID (5 or higher)
    let initial_value = Value::Int(42);  // Initial value to be captured
    global_scope.add(capture_id, initial_value.clone());

    // Step 2: Add another variable that will be returned as a reference in the match statement
    let return_id = 6; // Return ID >= 5 for realistic testing
    let return_value = Value::String(GcPointer::new("Captured Reference".to_string()));  // Captured value
    global_scope.add(return_id, return_value.clone());

    // Step 3: Define a MatchStatment for `match fn` that captures the value at `capture_id`
    let match_stmt = MatchStatment {
        arms: vec![
            MatchCond::Literal(Value::Int(1)),
            MatchCond::Literal(Value::Int(2)),
            MatchCond::Literal(Value::Int(42)),  // Match against the initial value at capture_id
        ],
        vals: vec![
            Block::new_simple(LazyVal::Ref(return_id)),  // Return reference to the captured value
            Block::new_simple(LazyVal::Terminal(Value::String(GcPointer::new("Two".to_string())))),
            Block::new_simple(LazyVal::Ref(return_id)),  // Return reference to captured value again
        ],
        debug_span: Span::new(0, 0),
    };

    // Step 4: Create the `match fn` as a normal value, no wrapping
    let lazy_match_fn = LazyVal::MakeMatchFunc(LazyMatch::new(match_stmt));

    // Step 5: Evaluate the `match fn` (this implicitly captures the variables)
    let handle = match lazy_match_fn.eval(&global_scope).unwrap() {
        ValueRet::Local(Value::Func(f)) => f,  // Ensure we get the function handle
        _ => panic!("Expected a function handle"),
    };

    // Step 6: Modify the variable in the global scope after the closure has been created
    let new_value = Value::Int(100);  // New value after modification
    global_scope.add(capture_id, new_value.clone());

    // Step 7: Call the closure with the captured value (42), which should return the old captured value
    let result = handle.eval(vec![initial_value]).unwrap();

    // Step 8: Check that the result is still the old captured value and not the modified one
    assert_eq!(result, return_value);

    // Confirm that the global scope holds the modified value, not the old one
    assert_eq!(global_scope.get(capture_id), Some(new_value));
}

#[test]
fn test_global_function_recursive_call() {
    // Initialize the string table and get IDs for `a`, `b`, and `x`
    let mut string_table = StringTable::new();
    let a_id = string_table.get_id("a");
    let b_id = string_table.get_id("b");
    let x_id = string_table.get_id("x");

    // Create the global scope and add functions before leaking it
    let mut global_scope = GlobalScope::default();

    // Define function `b`: call `a(0)`
    let b_sig = FuncSig { arg_ids: vec![x_id] }; // signature expects one argument (x)
    let b_block = Block::new(vec![
        Statment::Return(ScopeRet::Local(LazyVal::FuncCall(Call {
            called: Box::new(LazyVal::Ref(a_id)), // call function `a`
            args: vec![LazyVal::Terminal(Value::Int(0))], // with argument 0
            debug_span: Span::new(0, 1),
        }))),
    ]);

    // Add function `b` to global scope
    global_scope.add(b_id, b_block, b_sig).unwrap();

    // Define function `a`: match on the input `x`
    // If `x == 0`, return 10. Otherwise, call `b(0)`.
    let a_block = Block::new(vec![
        Statment::Return(ScopeRet::Local(LazyVal::Match {
            var: Box::new(LazyVal::Ref(x_id)), // `x` is passed as argument
            statment: MatchStatment::new(
                vec![
                    MatchCond::Literal(Value::Int(0)), // match on x == 0
                    MatchCond::Any,                    // default match
                ],
                vec![
                    // If `x == 0`, return 10
                    Block::new(vec![Statment::Return(ScopeRet::Local(LazyVal::Terminal(Value::Int(10))))]),
                    // Else, call `b(0)`
                    Block::new(vec![Statment::Return(ScopeRet::Local(LazyVal::FuncCall(Call {
                        called: Box::new(LazyVal::Ref(b_id)), // call function `b`
                        args: vec![LazyVal::Terminal(Value::Int(0))], // with argument 0
                        debug_span: Span::new(0, 1),
                    })))]),
                ],
                Span::new(0, 1),
            ),
        })),
    ]);

    let a_sig = FuncSig { arg_ids: vec![x_id] }; // signature expects one argument (x)
    // Add function `a` to global scope
    global_scope.add(a_id, a_block, a_sig).unwrap();

    // Leak the global scope
    let global_scope = Box::leak(Box::new(global_scope));

    // Now, retrieve `a` from the global scope and call it with argument 1 (or any non-zero value)
    let a_func = global_scope.get(a_id).unwrap();
    let result = if let Value::Func(f) = a_func {
        f.eval(vec![Value::Int(1)]).unwrap() // call `a(1)`
    } else {
        panic!("Expected a function");
    };

    // Check that the result is 10 (via the recursive call through `b`)
    assert_eq!(result, Value::Int(10));
}
