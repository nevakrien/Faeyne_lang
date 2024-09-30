#![allow(dead_code)]
// use core::cell::RefCell;
// use core::ptr::NonNull;
use ast::id::*;
use ast::get_id;
use std::rc;
use core::cell::Cell;
use crate::basic_ops::call_string;
use crate::basic_ops::nerfed_to_string;
use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry;


use std::rc::Rc;
use codespan::Span;

use std::fmt;
use std::ptr;

pub use crate::basic_ops::{is_equal};
use crate::reporting::*;

// #[derive(Debug, Clone , PartialEq)]
// pub struct Context {
//     recursion_size:Cell<usize>,
// }

#[derive(Debug, Clone , PartialEq)]
#[derive(Default)]
pub struct GlobalScope<'ctx> {
    vars: HashMap<usize, PreGlobalFunc<'ctx>>,
}


impl<'ctx> GlobalScope<'ctx> {
    pub fn get<'x : 'ctx>(&'x self, id: usize) -> Option<Value<'x>>  {
        let pre = self.vars.get(&id)?;
        pre.global.set(Some(self));
        Some(Value::Func(FunctionHandle::StaticDef(
            GlobalFunc {
                pre,
                //global:self
            },
        )))
    }

    pub fn add(&mut self, id: usize, block: Block<'ctx>, sig: FuncSig) -> Result<(), ErrList>{
        //horible attempt to just measure do not keep this mess
        if let std::collections::hash_map::Entry::Vacant(e) = self.vars.entry(id) {
            e.insert(PreGlobalFunc{ sig, inner:block,global:None.into()}.into());//,global:ptr });
            Ok(())
        } else {
            // TODO: Handle this case with more complex behavior when adding multiple catching patterns
            // For now, we mark this as a TODO to indicate the behavior needs more consideration.
            Err(Error::UnreachableCase(UnreachableCase {
                name: id,
                sig,
            })
            .to_list())
        }
    }



    pub fn make_subscope<'a : 'ctx>(&'a self,depth:usize) -> Result<VarScope<'a, 'a>,ErrList> {
        VarScope::new(Scopble::Global(self),depth) 
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Scopble<'ctx, 'parent>
where
    'ctx: 'parent,
{   
    Global(&'ctx GlobalScope<'ctx>),
    SubScope(&'parent VarScope<'ctx, 'parent>),
    Static(&'parent ClosureScope<'ctx>),
}

impl<'ctx, 'parent> Scopble<'ctx, 'parent>
where
    'ctx: 'parent,
{
    pub fn get(&self, id: usize) -> Option<Value<'ctx>> {
        match self {
            Scopble::Global(s) => s.get(id),
            Scopble::SubScope(s) => s.get(id),
            Scopble::Static(s) => s.get(id),
        }
    }

    pub fn get_root(&self) -> Scopble<'ctx, 'parent> {
        match self {
            Scopble::SubScope(s) => s.parent.get_root(),
            Scopble::Static(s) => Scopble::Static(s),
            Scopble::Global(s) => Scopble::Global(s),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarScope<'ctx, 'parent>
where
    'ctx: 'parent,
{
    parent: Scopble<'ctx, 'parent>,
    vars: HashMap<usize, Value<'ctx>>,
    depth:usize,
}

impl<'ctx, 'parent> VarScope<'ctx, 'parent>
where
    'ctx: 'parent,
{
    pub fn new<'p:'parent>(parent: Scopble<'ctx, 'p>,depth:usize) -> Result<Self,ErrList> {
        let depth = depth+1;
        if depth >= MAX_RECURSION {
            return Err(Error::Recursion(RecursionError{depth}).to_list());
        } 
        Ok(VarScope {
            parent,
            vars: HashMap::new(),
            depth,
        })
    }

    pub fn make_subscope(&self) -> Result<VarScope<'ctx, '_>,ErrList> {
        let depth = self.depth+1;
        if depth >= MAX_RECURSION {
            return Err(Error::Recursion(RecursionError{depth}).to_list());
        } 
        Ok(VarScope {
            parent: Scopble::SubScope(self),
            vars: HashMap::new(),
            depth,
        })
    }

    pub fn add(&mut self, id: usize, val: Value<'ctx>) {
        self.vars.insert(id, val);
    }

    pub fn get(&self, id: usize) -> Option<Value<'ctx>> {
        match self.vars.get(&id) {
            Some(v) => Some(v.clone()),
            None => self.parent.get(id),
        }
    }
}


#[test]
fn test_scope_lifetimes(){
    let r = ClosureScope::new();
	let g = VarScope::new(Scopble::Static(&r),1).unwrap();
	let mut a = g.make_subscope().unwrap();
	{
		let _c = a.make_subscope();
	}
	let _d = &mut a;
}


// #[derive(Debug,PartialEq)]
pub struct ClosureScope<'ctx> {
    vars : HashMap<usize,Value<'ctx>>,
    allowed_escapes : HashSet<usize>,
    self_ref : Cell<Option<GcPointer< Func<'ctx>>>> 
}


// Manually implement Debug for ClosureScope
impl<'ctx> fmt::Debug for ClosureScope<'ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Attempt to upgrade the weak pointer for debug output, panic if upgrade fails
        let strong_self_ref = self.self_ref.replace(None);//get_strong_ref(&self.self_ref);

        let ans = f.debug_struct("ClosureScope")
            .field("vars", &self.vars)
            .field("allowed_escapes", &self.allowed_escapes)
            .field("self_ref", &strong_self_ref)
            .finish();

        self.self_ref.set(strong_self_ref);
        ans
    }
}

// Manually implement PartialEq for ClosureScope
impl<'ctx> PartialEq for ClosureScope<'ctx> {
    fn eq(&self, other: &Self) -> bool {
        // Compare vars and allowed_escapes first
        if self.vars != other.vars || self.allowed_escapes != other.allowed_escapes {
            return false;
        }

        // Compare the self_ref by upgrading both weak pointers, panic if upgrade fails
        let self_ref = self.self_ref.replace(None);//get_strong_ref(&self.self_ref);
        let other_ref = other.self_ref.replace(None);

        // Compare the strong references
        let ans = match (&self_ref,&other_ref){
            (None,None) => true,
            (None,Some(_)) | (Some(_),None)=> false,
            (Some(a),Some(b)) => GcPointer::ptr_eq(a,b)
           
        };

        self.self_ref.set(self_ref);
        other.self_ref.set(other_ref);
        ans
    }
}

impl<'ctx,'call> Default for ClosureScope<'ctx> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'ctx,'call> ClosureScope<'ctx> {
    pub fn new() -> Self {
        let mut allowed_escapes = HashSet::new();
        allowed_escapes.insert(get_id!("self"));
        ClosureScope{vars: HashMap::new(),allowed_escapes,self_ref:None.into()}
    }

    pub fn get(&self,id:usize) -> Option<Value<'ctx>> {
        match self.vars.get(&id){
            Some(x)=>Some(x.clone()),
            None=> if id==get_id!("self"){
                let ans = self.self_ref.replace(None).expect("closures allways hold ref to self");
                self.self_ref.set(Some(ans.clone()));
                Some(Value::Func(FunctionHandle::Lambda(ans)))
            } else{
                None
            }
        }
    }

    pub fn maybe_add(&mut self,id : usize ,outer_scope: &VarScope<'ctx,'_>) -> Result<(),ErrList> {
        match self.vars.entry(id){
            Entry::Occupied(_) => Ok(()), 
            Entry::Vacant(spot) => {
                match outer_scope.get(id) {
                    // None => Err(Error::Missing(UndefinedName{id}).to_list()),
                    // None => Ok(()),//for debuging
                    Some(v) => {spot.insert(v.clone()); Ok(())}
                    None => if self.allowed_escapes.contains(&id) {Ok(())} 
                        else {Err(Error::Missing(UndefinedName{id}).to_list())}

                }
            }
        }
    }

    pub fn add_args(&mut self,sig:&FuncSig){
        for id in sig.arg_ids.iter() {
            self.allowed_escapes.insert(*id);

        }
    }


    pub fn make_subscope(&self,depth:usize) -> Result<VarScope<'ctx,'_>,ErrList> {
        VarScope::new(Scopble::Static(self),depth)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RetType{
    Local,
    Unwind
}

#[derive(Debug, PartialEq, Clone)]
pub struct GenericRet<T> {
    value: T,
    ret: RetType,
}

impl<T :Clone + PartialEq> GenericRet<T> {
    pub fn new(value: T, ret: RetType) -> Self {
        GenericRet { value, ret }
    }

    pub fn new_local(value: T) -> Self {
        GenericRet {
            value,
            ret: RetType::Local,
        }
    }

    pub fn new_unwind(value: T) -> Self {
        GenericRet {
            value,
            ret: RetType::Unwind,
        }
    }

    pub fn as_local(self) -> Self {
        GenericRet {
            value: self.value,
            ret: RetType::Local,
        }
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub fn map<U :Clone + PartialEq, F>(self, f: F) -> GenericRet<U>
    where
        F: FnOnce(T) -> U,
    {
        GenericRet {
            value: f(self.value),
            ret: self.ret,
        }
    }
}


// // Define the types for ScopeRet and ValueRet using the generic enum

pub type ScopeRet<'ctx> = GenericRet<LazyVal<'ctx>>;
pub type ValueRet<'ctx> = GenericRet<Value<'ctx>>;


impl<'ctx> ValueRet<'ctx> {
    fn to_unwind(self) -> Self {
        let value: Value = self.into_inner();
        ValueRet::new(value, RetType::Unwind)
    }
}

impl<'ctx> From<Value<'ctx>> for ValueRet<'ctx> {
    fn from(value: Value<'ctx>) -> Self {
        ValueRet::new_local(value)
    }
}

impl<'ctx> From<ValueRet<'ctx>> for Value<'ctx> {
    fn from(ret: ValueRet<'ctx>) -> Self {
        ret.into_inner()
    }
}


impl<'ctx> From<ScopeRet<'ctx>> for LazyVal<'ctx> {
    fn from(ret: ScopeRet<'ctx>) -> Self {
        ret.into_inner()
    }
}

pub type GcPointer<T> = Rc<T>;
pub type WeakGcPointer<T> = rc::Weak<T>;

pub fn get_strong_ref<'ctx>(
    cell: &Cell<Option<WeakGcPointer<Func<'ctx>>>>
) -> Option<GcPointer<Func<'ctx>>> {
    // Swap out the current value with None temporarily
    let old_value = cell.take();

    // Attempt to upgrade the weak pointer to a strong reference
    let strong_ref = old_value.as_ref().and_then(|weak| weak.upgrade());

    // Put the old value back into the cell
    cell.set(old_value);

    strong_ref
}

#[derive(Debug,PartialEq,Clone)]
pub enum LazyVal<'ctx>{
    //this represents a value before computation
    //however early return complicate things 
    //so we can actually return from the parent scope
    //sometimes evaluating a var causes an excepsion
    
    MakeMatchFunc(LazyMatch<'ctx>),
    MakeFunc(LazyFunc<'ctx>),

    Terminal(Value<'ctx>),
    Ref(usize),
    FuncCall(Call<'ctx>),
    //this is scoped for now but in some cases that makes no sense
    Match{var: Box<LazyVal<'ctx>>,statment: MatchStatment<'ctx>}
}

impl<'ctx> LazyVal<'ctx> {
    pub fn eval<'parent>(
        &self,
        scope: &mut VarScope<'ctx, 'parent>
    ) -> Result<ValueRet<'ctx>, ErrList>
    where
        'ctx: 'parent,
    {
        match self {
            LazyVal::Terminal(v) => Ok(ValueRet::new_local(v.clone())),
            LazyVal::Ref(id) => match scope.get(*id) {
                None => Err(Error::Missing(UndefinedName { id: *id }).to_list()),
                Some(v) => Ok(ValueRet::new_local(v.clone())), //this line is slow seems like a cach miss
            },
            LazyVal::FuncCall(call) => call.eval(scope),
            LazyVal::Match { var, statment } => {
                let ValueRet{value,ret} = var.eval(scope)?;
                match ret {
                    RetType::Unwind => Ok(ValueRet::new_unwind(value)),
                    RetType::Local => statment.eval(value, scope),
                }
            },
            LazyVal::MakeFunc(lf) => lf
                .eval(scope)
                .map(|f| ValueRet::new_local(Value::Func(FunctionHandle::Lambda(GcPointer::new(f))))),
            LazyVal::MakeMatchFunc(x) => x
                .eval(scope)
                .map(|f| ValueRet::new_local(Value::Func(FunctionHandle::Lambda(GcPointer::new(f))))),
        }
    }


    pub fn add_to_closure(&self,scope: &VarScope<'ctx, '_>,closure : &mut ClosureScope<'ctx>) -> Result<(),ErrList> {
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
pub enum Statment<'ctx>{
    Assign(usize, LazyVal<'ctx>),
    Call(Call<'ctx>),
    Match((LazyVal<'ctx>,MatchStatment<'ctx>)),
    Return(ScopeRet<'ctx>),
}

impl<'ctx> Statment<'ctx>{
    pub fn add_to_closure(&self,scope: &VarScope<'ctx,'_>,closure : &mut ClosureScope<'ctx>) -> Result<(),ErrList>{
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
pub struct LazyFunc<'ctx>{
    pub sig:FuncSig,
    pub inner:Block<'ctx>,
    pub debug_span:Span,
}

impl<'ctx> LazyFunc<'ctx> {
    pub fn new(sig : FuncSig,inner : Block<'ctx>,debug_span:Span) -> Self {
       LazyFunc{
            sig,inner,debug_span
       }
    }
    pub fn eval(&self,scope: &VarScope<'ctx, '_>) -> Result<Func<'ctx>,ErrList> {
        let mut closure : ClosureScope<'ctx>= ClosureScope::new();
        
        closure.add_args(&self.sig);

        match self.inner.add_to_closure(scope,&mut closure){
            Ok(_) =>Ok(Func {
                        sig:self.sig.clone(),
                        inner: self.inner.clone(),
                        closure
                    }),
            Err(err) => Err(Error::Stacked(
                InternalError{
                    err,
                    span:self.debug_span,
                    message:"When defining Closure".to_string()
                }).to_list())
        }  
        
    }

    pub fn add_to_closure(&self,scope: &VarScope<'ctx, '_>,closure : &mut ClosureScope<'ctx>) -> Result<(),ErrList> {
        self.inner.add_to_closure(scope,closure)
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct LazyMatch<'ctx>{
    inner:MatchStatment<'ctx>,
}

impl<'ctx> LazyMatch<'ctx> {
    pub fn new(inner : MatchStatment<'ctx>) -> Self {
       LazyMatch{
            inner
       }
    }
    pub fn eval(&self, scope: &VarScope<'ctx, '_>) -> Result<Func<'ctx>, ErrList> {
        let mut closure = ClosureScope::new();

        self.inner.add_to_closure(scope, &mut closure)?;

        //we are using some cursed ideas basically the 0 id will never be a varible name
        // thus using it as an argument is totally safe

        let statment = Statment::Return(
            ScopeRet::new_local(
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


    pub fn add_to_closure(&self,scope: &VarScope<'ctx, '_>,closure : &mut ClosureScope<'ctx>) -> Result<(),ErrList> {
        self.inner.add_to_closure(scope,closure)
    }
}


#[derive(Debug,Clone)]
pub struct GlobalFunc<'ctx> {
    pre:&'ctx PreGlobalFunc<'ctx>,
    // global: &'ctx GlobalScope<'ctx>,
}

impl<'ctx> PartialEq for GlobalFunc<'ctx>{
    fn eq(&self, other: &GlobalFunc<'ctx>) -> bool { std::ptr::eq(self.pre,other.pre) }
}

#[derive(Debug,PartialEq,Clone)]
pub struct PreGlobalFunc<'ctx> {
    sig:FuncSig,
    inner:Block<'ctx>,
    global: Cell<Option<&'ctx GlobalScope<'ctx>>>,
}

impl<'ctx> GlobalFunc<'ctx> {
    pub fn eval(&self,args: Vec<Value<'ctx>>,depth:usize) -> Result<Value<'ctx>,ErrList> {
        self.pre.sig.matches(&args)?;
        let mut scope = self.pre.global.get().unwrap().make_subscope(depth)?;
        for (i,a) in self.pre.sig.arg_ids.iter().enumerate(){
            scope.add(*a,args[i].clone());
        }
        
        self.pre.inner.eval(&mut scope).map(|x| x.into())
             
    }
}


#[derive(Debug,PartialEq)]
pub struct Func<'ctx> {
    sig:FuncSig,
    closure:ClosureScope<'ctx>,
    inner:Block<'ctx>,
}

// impl<'ctx> Clone for Func<'ctx>{
//     fn clone(&self) -> Self {
//         let closure = self.closure.clone();

//         Func{
//             sig:self.sig.clone(),
//             closure,
//             inner:self.inner.clone()
//         }
//     }
// }

impl<'ctx> Func<'ctx> {
    // pub fn eval(&self,args: Vec<Value<'ctx>>) -> Result<Value<'ctx>,ErrList> {
    //     // self.closure.self_ref.set(Some(self.into()));
    //     let _parent = get_strong_ref(&self.closure.self_ref);

    //     self.sig.matches(&args)?;
    //     let mut scope = self.closure.make_subscope();
    //     for (i,a) in self.sig.arg_ids.iter().enumerate(){
    //         scope.add(*a,args[i].clone());
    //     }
        
    //     self.inner.eval(&mut scope).map(move |x| x.into()) 
    // }

    pub fn eval_gc(strong:GcPointer<Self>,args: Vec<Value<'ctx>>,depth:usize) -> Result<Value<'ctx>,ErrList> {
        strong.sig.matches(&args)?;
        let mut scope = strong.closure.make_subscope(depth)?;
        
        for (i,a) in strong.sig.arg_ids.iter().enumerate(){
            scope.add(*a,args[i].clone());
        }
        
        strong.closure.self_ref.set(Some(strong.clone()));
        let ans = strong.inner.eval(&mut scope).map(move |x| x.into()); 
        strong.closure.self_ref.set(None);
        ans

    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct FuncSig{
    pub arg_ids: Vec<usize>, //for now its all simple
}

impl FuncSig {
   pub fn matches(&self,args: &[Value]) -> Result<(),ErrList> {
        if args.len() == self.arg_ids.len() {
            Ok(())
        } else {
            Err(Error::Sig(SigError{}).to_list())
        }
   }

   // pub fn add_to_closure<'ctx>(&self,scope: &VarScope<'ctx, '_>,closure : &mut ClosureScope<'ctx>) -> Result<(),ErrList> {
   //      for i in self.arg_ids {
   //          closure.add(LazyVal::Ref(*i))
   //      }
   //      Ok(())
   //  }
}

#[derive(Debug,PartialEq,Clone)]
pub struct Block<'ctx>{
    code: Vec<Statment<'ctx>>,
}


impl<'ctx> Block <'ctx>{
    pub fn new(code : Vec<Statment<'ctx>>) -> Self {
        Block{code}
    }

    pub fn new_simple(val :LazyVal<'ctx>) -> Self {
        Block {
            code: vec![Statment::Return(GenericRet::new_local(val))]
        }
    }

    pub fn eval<'parent>(&self,scope: &mut VarScope<'ctx, 'parent>)-> Result<ValueRet<'ctx>,ErrList> 
    where 'ctx :'parent,
    {
        // let mut scope = parent_scope.make_subscope();

        for s in self.code.iter() {
            match s {
                Statment::Return(a) => match a.ret{
                    RetType::Local => {return a.value.eval(scope);},
                    RetType::Unwind=> {return a.value.eval(scope).map(|x| x.to_unwind());},
                },
                Statment::Assign(id,a) =>{
                    let ValueRet{value,ret} = a.eval(scope)?;
                    match ret{
                        RetType::Local => {scope.add(*id,value);},
                        RetType::Unwind => {return Ok(ValueRet::new_unwind(value));}
                    }
                    
                },
                Statment::Call(v) => {
                    let ValueRet{value,ret} = v.eval(scope)?;
                    match ret {
                        RetType::Local => {},
                        RetType::Unwind => {return Ok(ValueRet::new_unwind(value));}
                    }
                },
                Statment::Match((val,statment)) => {
                    let ValueRet{value:x,ret} = val.eval(scope)?;
                    match ret {
                        RetType::Local => {},
                        RetType::Unwind =>{return Ok(ValueRet::new_unwind(x));},
                    };
                    let ValueRet{value:y,ret} = statment.eval(x,scope)?;
                    match ret {
                        RetType::Local => {},
                        
                        RetType::Unwind => {return Ok(ValueRet::new_unwind(y));},
                    }
                },
            }
        }

        Ok(ValueRet::new_local(Value::Nil))
    }

    pub fn add_to_closure(&self,scope: &VarScope<'ctx,'_>,closure : &mut ClosureScope<'ctx>) -> Result<(),ErrList>{
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
pub enum MatchCond<'ctx> {
    Literal(Value<'ctx>),
    Any
}

impl<'ctx> MatchCond<'ctx> {
    pub fn matches(&self,v: &Value<'ctx>) -> bool{
        match self {
            MatchCond::Any => true,
            MatchCond::Literal(x) => is_equal(v,x),
        }
    }
    pub fn add_to_closure(&self,_scope: &VarScope<'ctx,'_>,_closure : &mut ClosureScope) -> Result<(),ErrList>{
        Ok(())
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct MatchStatment<'ctx> {
    arms: Vec<MatchCond<'ctx>>,
    vals: Vec<Block<'ctx>>,
    debug_span: Span,
}

impl<'ctx> MatchStatment<'ctx> {
    pub fn new (arms: Vec<MatchCond<'ctx>>,vals: Vec<Block<'ctx>>,debug_span: Span,) -> Self {
        MatchStatment{arms,vals,debug_span}
    }
    pub fn eval(&self, x:Value<'ctx> ,scope: &mut VarScope<'ctx, '_>) -> Result<ValueRet<'ctx>,ErrList> {
        for (i,a) in self.arms.iter().enumerate() {
            if a.matches(&x) {
                return self.vals[i].eval(scope);
            }
        }
        Err(Error::Match(MatchError{span:self.debug_span}).to_list())
    }

    pub fn add_to_closure(&self,scope: &VarScope<'ctx, '_>,closure : &mut ClosureScope<'ctx>) -> Result<(),ErrList> {
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

pub type DynFFI<'ctx> = dyn Fn(Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList>;

#[derive(Clone)]
pub enum FunctionHandle<'ctx> {
    FFI(fn(Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList>),
    StateFFI(&'ctx DynFFI<'ctx>),
    DataFFI(GcPointer<DynFFI<'ctx>>),
    // MutFFI(Box<dyn FnMut(Vec<Value>) -> Result<Value, ErrList>>), // New FnMut variant
    StaticDef(GlobalFunc<'ctx>),
    Lambda(GcPointer<Func<'ctx>>),
}

impl<'ctx> PartialEq for FunctionHandle<'ctx> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FunctionHandle::FFI(f1), FunctionHandle::FFI(f2)) => f1 == f2,
            (FunctionHandle::StateFFI(f1), FunctionHandle::StateFFI(f2)) => ptr::eq(f1, f2),
            (FunctionHandle::DataFFI(f1), FunctionHandle::DataFFI(f2)) => GcPointer::ptr_eq(f1, f2),
            // (FunctionHandle::MutFFI(f1), FunctionHandle::MutFFI(f2)) => Box::ptr_eq(f1, f2),
            (FunctionHandle::StaticDef(f1), FunctionHandle::StaticDef(f2)) => f1 == f2,
            (FunctionHandle::Lambda(f1), FunctionHandle::Lambda(f2)) => GcPointer::ptr_eq(f1, f2),
            _ => false,
        }
    }
}

impl<'ctx> fmt::Debug for FunctionHandle<'ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionHandle::FFI(func) => write!(f, "{:?}", func),
            FunctionHandle::StateFFI(func) => {
                write!(f, "StateFFI({:p})", func as *const dyn Fn(Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList>)
            },
            FunctionHandle::DataFFI(func) => {
                write!(f, "DataFFI({:p})", GcPointer::as_ptr(func))
            },
            // FunctionHandle::MutFFI(func) => {
            //     write!(f, "MutFFI({:p})", Box::as_ptr(func))
            // },
            FunctionHandle::StaticDef(func) => write!(f, "{:?}", func),
            FunctionHandle::Lambda(func) => write!(f, "{:?}", func),
        }
    }
}

impl<'ctx> FunctionHandle<'ctx> {
    pub fn eval(self, args: Vec<Value<'ctx>>,depth:usize) -> Result<Value<'ctx>, ErrList> {
        match self {
            FunctionHandle::FFI(f) => f(args),
            FunctionHandle::StateFFI(f) => f(args),
            FunctionHandle::DataFFI(f) => f(args),
            // FunctionHandle::MutFFI(mut f) => f(args),
            FunctionHandle::StaticDef(f) => f.eval(args,depth),
            FunctionHandle::Lambda(f) => Func::eval_gc(f,args,depth),//f.eval(args),
        }
    }
}


#[derive(Debug,PartialEq,Clone)]
pub struct Call<'ctx>{
    pub called: Box<LazyVal<'ctx>>,
    pub args: Vec<LazyVal<'ctx>>,
    pub debug_span: Span
}

impl<'ctx> Call<'ctx> {
    pub fn eval(&self,scope: &mut VarScope<'ctx, '_>) -> Result<ValueRet<'ctx>,ErrList> {
        let ValueRet{ret,value} =  self.called.eval(scope)?;
        let handle_res = match ret {
            RetType::Unwind => {return Ok(ValueRet::new_unwind(value));}
            RetType::Local => match value {
                Value::Func(f) => Ok(f),
                Value::String(s) => Err(s),
                _ => {return Err(Error::NoneCallble(NoneCallble{span:self.debug_span,value:nerfed_to_string(&value)}).to_list());}
            }
        };

        let mut arg_values = Vec::with_capacity(self.args.len());
        for a in self.args.iter() {
            match a.eval(scope) {
                Err(e) => {return Err(e);},
                Ok(ValueRet{value,ret}) => {match ret {
                    RetType::Local => {arg_values.push(value);},
                    RetType::Unwind => {return Ok(ValueRet::new_local(value));},
                }}
            };
        }
        match handle_res{
            Ok(handle) => match handle.eval(arg_values,scope.depth+1) {
                Ok(x) => Ok(x.into()),
                Err(err) => Err(Error::Stacked(
                    InternalError{
                        err,
                        span:self.debug_span,
                        message:"When calling Function".to_string()
                    }
                ).to_list())
            },

            Err(s) => call_string(s,arg_values).map(|x| x.into())
        }
        
    }

    pub fn add_to_closure(&self,scope: &VarScope<'ctx,'_>,closure : &mut ClosureScope<'ctx>) -> Result<(),ErrList> {
        let mut ans = self.called.add_to_closure(scope,closure);
        for a in self.args.iter() {
            ans = append_err_list(ans,
                a.add_to_closure(scope,closure)
            );
        }
        ans
    }
}

// #[repr(align(16))]
//seems that 
#[derive(Debug,PartialEq,Clone)]
pub enum Value<'ctx> {
	Nil,
	Bool(bool),
	Atom(usize),
	String(GcPointer<String>),
	Int(i64),
	Float(f64),
	Func(FunctionHandle<'ctx> ),
}



// #[cfg(test)]
// use std::cell::RefCell;

#[cfg(test)]
use ast::ast::StringTable;






#[test]
fn test_system_state_ffi() {
    use std::cell::RefCell;     
    
    // `ffi_println` logs the arguments into `log_print` and returns `Value::Nil`
    fn ffi_println<'ctx>(
        log_print: GcPointer<RefCell<Vec<Vec<Value<'ctx>>>>>,
        args: Vec<Value<'ctx>>,
    ) -> Result<Value<'ctx>, ErrList> {
        log_print.borrow_mut().push(args.clone());
        Ok(Value::Nil)
    }

    // `ffi_system` logs the arguments into `log_system` and returns a function handle to `ffi_println`
    fn ffi_system<'ctx>(
        log_system: GcPointer<RefCell<Vec<Vec<Value<'ctx>>>>>,
        print_func: * const DynFFI<'ctx>,
        args: Vec<Value<'ctx>>,
    ) -> Result<Value<'ctx>, ErrList> {
        log_system.borrow_mut().push(args.clone());

        

        unsafe {Ok(Value::Func(FunctionHandle::StateFFI(&*print_func)))}
    }

    // Helper function to constrain the closure with a lifetime
    fn constrain_lifetime<'ctx, F>(f: F) -> F
    where
        F: Fn(Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> + 'ctx,
    {
        f
    }


    //we will use these for freeing later
    let  root_ptr;
    let  print_ptr;
    let  system_ptr;

    

    {    
        // Step 1: Create mutable buffers for logging (no static references)
        
        let root = Box::leak(Box::new(GlobalScope::default()));
        root_ptr = root as *mut _;

        
        
        let log_print: GcPointer<RefCell<Vec<Vec<Value>>>> = GcPointer::new(RefCell::new(Vec::new()));
        let log_system: GcPointer<RefCell<Vec<Vec<Value>>>> = GcPointer::new(RefCell::new(Vec::new()));
        


        // Step 3: Initialize the scope (no leaking, just normal scoped variables)
        let mut string_table = StringTable::new();
        let system_name = string_table.get_id("system");
        let println_name = string_table.get_id(":println");


        // Clone GcPointer before moving into the closure
        let log_system_clone = log_system.clone();
        let log_print_clone = log_print.clone();

        // Box the `ffi_println` closure and return a mutable reference to it
        let println_ref =
            Box::leak(Box::new(constrain_lifetime(move |a: Vec<Value<'_>>| -> Result<Value<'_>, ErrList> {
                ffi_println(log_print_clone.clone(), a)
            })));
        print_ptr = println_ref as *mut _; 

        // Add the `ffi_system` closure to the scope (returning a mutable reference instead of leaking)
        let system_ref = Box::leak(Box::new(constrain_lifetime(move |args: Vec<Value<'_>>| -> Result<Value<'_>, ErrList> {
            ffi_system(
                log_system_clone.clone(),
                print_ptr,
                 args)
        })));

        system_ptr = system_ref as *mut _;
        
        let mut scope = root.make_subscope(0).unwrap();
        scope.add(system_name, Value::Func(FunctionHandle::StateFFI(system_ref)));

        // Step 4: Create the call structures
        // Inner call for accessing the `system` variable from the scope
        let system_call = Call {
            called: Box::new(LazyVal::Ref(system_name)),
            args: vec![LazyVal::Terminal(Value::Atom(println_name))],
            debug_span: Span::new(0, 1),
        };

        // Outer call for `system(:println)("hello world")`
        let outer_call = Call {
            called: Box::new(LazyVal::FuncCall(system_call)),
            args: vec![LazyVal::Terminal(Value::String(GcPointer::new(
                "hello world".to_string(),
            )))],
            debug_span: Span::new(0, 1),
        };

        // Step 5: Evaluate the outer call
        let ans = outer_call.eval(&mut scope).unwrap();
        assert_eq!(ans, ValueRet::new_local(Value::Nil));

        // Step 6: Validate that the logs were updated correctly
        assert_eq!(log_system.borrow().len(), 1);
        assert_eq!(log_system.borrow()[0], vec![Value::Atom(println_name)]);

        assert_eq!(log_print.borrow().len(), 1);
        assert_eq!(
            log_print.borrow()[0],
            vec![Value::String(GcPointer::new("hello world".to_string()))]
        );
    }
    //c style freeing
    unsafe {
        {
        _ = Box::from_raw(system_ptr);

        }
        {
        _ = Box::from_raw(print_ptr);

        }
        {
        _ = Box::from_raw(root_ptr);
        }
    }
}




