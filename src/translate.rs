use crate::ast::*;
use crate::ir;
use codespan::Span;
use crate::ir::{LazyVal,GcPointer,ScopeRet,GlobalScope};
use crate::basic_ops::get_buildin_function;
use crate::reporting::*;

pub fn translate_program<'ctx>(v:Vec<OuterExp>,table: &StringTable<'ctx>) -> Result<Box<GlobalScope<'ctx>>,ErrList>{
	let mut scope = Box::new(GlobalScope::default());
	let mut e = Ok(());

	for s in v.into_iter(){
		let e1 = match s {
			OuterExp::FuncDec(f) => scope.add_func_dec(f,table),
			OuterExp::ImportFunc(i) => scope.add_import(i,table),
		};

		e=append_err_list(e,e1);
	}
	e?;
	Ok(scope)
}

trait AddToGlobalScope<'ctx> {
    fn add_func_dec(&mut self, func_dec: FuncDec, table: &StringTable<'ctx>) -> Result<(), ErrList>;
    fn add_import(&mut self, import_func: ImportFunc,table: &StringTable<'ctx>) -> Result<(), ErrList>; // TODO handler for imports
}

impl<'ctx> AddToGlobalScope<'ctx> for GlobalScope<'ctx> {
    fn add_func_dec(&mut self, func_dec: FuncDec, table: &StringTable<'ctx>) -> Result<(), ErrList> {
        // Function name and signature handling as before
        let func_name_id = func_dec.sig.name;
        let func_sig = ir::FuncSig { arg_ids: func_dec.sig.args };
        
        // Translate the function body using table
        let func_body = func_dec.body.translate(table)?;
        
        // Add function to global scope
        self.add(func_name_id, func_body, func_sig)?;
        Ok(())
    }

    fn add_import(&mut self, _import_func: ImportFunc, _table: &StringTable<'ctx>) -> Result<(), ErrList> {
        todo!("Handling of imports is not yet implemented.");
    }
}


pub trait Translate<'ctx, Output> {
    fn translate(self, table: &StringTable<'ctx>) -> Result<Output,ErrList>;
}


impl<'ctx> Translate<'ctx,LazyVal<'ctx>> for FValue {
	fn translate(self, table: &StringTable<'ctx>) -> Result<LazyVal<'ctx>, ErrList> {
		let val : Value = self.into();
		val.translate(table)
	}
}

impl<'ctx> Translate<'ctx,LazyVal<'ctx>> for Value {
	fn translate(self, table: &StringTable<'ctx>) -> Result<LazyVal<'ctx>, ErrList> {
		match self {
			Value::Variable(id) => Ok(LazyVal::Ref(id)),
			Value::Int(x) => Ok(LazyVal::Terminal(ir::Value::Int(x))),
			Value::Float(x) => Ok(LazyVal::Terminal(ir::Value::Float(x))),
			Value::Bool(x) => Ok(LazyVal::Terminal(ir::Value::Bool(x))),
			Value::Atom(x) => Ok(LazyVal::Terminal(ir::Value::Atom(x))),
			
			Value::Nil => Ok(LazyVal::Terminal(ir::Value::Nil)),
			Value::FuncCall(call) =>Ok(LazyVal::FuncCall(call.translate(table)?)),


			Value::String(x) => {
				let r = table.get_string(x).unwrap();
				let s = r[1..r.len()-1].to_string();

				Ok(LazyVal::Terminal(ir::Value::String(
					GcPointer::new(s)
				)))
			},
			


			Value::BuildIn(op) => Ok(LazyVal::Terminal(
				ir::Value::Func(
				ir::FunctionHandle::FFI(
					get_buildin_function(op)
				)
			))),

			Value::Lambda(f) => Ok(LazyVal::MakeFunc(f.translate(table)?)),
			Value::MatchLambda(f) => Ok(LazyVal::MakeMatchFunc(f.translate(table)?)),

			Value::Match(m) => {
				let (var_in,statment)= m.translate(table)?;
				let var = Box::new(var_in);
				Ok(LazyVal::Match{var,statment})
			},
			Value::SelfRef(span) => Err(Error::IllegalSelfRef(IllegalSelfRef{span}).to_list()),
		}
	}
}

impl<'ctx> Translate<'ctx,ir::Call<'ctx>>  for FunctionCall {
	fn translate(self, table: &StringTable<'ctx>) -> Result<ir::Call<'ctx>,ErrList> {
		let called = Box::new(self.name.translate(table)?);
		let mut args = Vec::with_capacity(self.args.len());
		for x in  self.args.into_iter() {
			args.push(x.translate(table)?);
		}
		Ok(ir::Call{
			called,
			args,
			debug_span: self.debug_span
		})
	}
}



impl<'ctx> Translate<'ctx,ir::Block<'ctx>>  for FuncBlock {
	fn translate(self, table: &StringTable<'ctx>) -> Result<ir::Block<'ctx>,ErrList> {
		let mut ans = Vec::with_capacity(self.body.len()+1);
		for s in self.body.into_iter() {
			ans.push(s.translate(table)?);
		}
		ans.push(ir::Statment::Return (match self.ret {
			None => ir::GenericRet::new_local(LazyVal::Terminal(ir::Value::Nil)),
			Some(r) => match r {
				Ret::Imp(x) => ScopeRet::new_local(x.translate(table)?),
				Ret::Exp(x) => ScopeRet::new_unwind(x.translate(table)?),
			},
		}));

		Ok(ir::Block::new(ans))
	}
}

impl<'ctx> Translate<'ctx,ir::MatchStatment<'ctx>>  for (Vec<MatchArm>,Span) {
	fn translate(self, table: &StringTable<'ctx>) -> Result<ir::MatchStatment<'ctx>,ErrList> {
		let mut conds = Vec::with_capacity(self.0.len());
		let mut vals = Vec::with_capacity(self.0.len());

		for a in self.0.into_iter() {
			conds.push(a.pattern.translate(table)?);
			vals.push(a.result.translate(table)?);
		}

		Ok(ir::MatchStatment::new(conds,vals,self.1))
	}
} 

impl<'ctx> Translate<'ctx,(LazyVal<'ctx>,ir::MatchStatment<'ctx>) >  for MatchStatment {
	fn translate(self, table: &StringTable<'ctx>) -> Result<(LazyVal<'ctx>, ir::MatchStatment<'ctx>), ErrList> {
		let var = self.val.translate(table)?;
		let statment = (self.arms,self.debug_span).translate(table)?;
		Ok((var, statment))
	}
}

impl<'ctx> Translate<'ctx,ir::LazyMatch<'ctx>>  for MatchLambda {
    fn translate(self, table: &StringTable<'ctx>) -> Result<ir::LazyMatch<'ctx>, ErrList> {
        let match_statement = (self.arms, self.debug_span).translate(table);
        Ok(ir::LazyMatch::new(match_statement?))
    }
}

impl<'ctx> Translate<'ctx,ir::LazyFunc<'ctx>>  for Lambda {
    fn translate(self, table: &StringTable<'ctx>) -> Result<ir::LazyFunc<'ctx>, ErrList> {
        let sig = ir::FuncSig { arg_ids: self.sig };
        let body = self.body.translate(table)?;
        Ok(ir::LazyFunc::new(sig, body,self.debug_span))
    }
}

impl<'ctx> Translate<'ctx,ir::Statment<'ctx>> for Statment {
	fn translate(self, table: &StringTable<'ctx>) -> Result<ir::Statment<'ctx>, ErrList> {
		match self {
			Statment::Assign(id,x) => Ok(ir::Statment::Assign(id,x.translate(table)?)),
			Statment::Call(f) => Ok(ir::Statment::Call(f.translate(table)?)),
			Statment::Match(m) => Ok(ir::Statment::Match(m.translate(table)?)),
		}
	}
}

impl<'ctx> Translate<'ctx,ir::MatchCond<'ctx>> for MatchPattern {
    fn translate(self, table: &StringTable<'ctx>) -> Result<ir::MatchCond<'ctx>, ErrList> {
        match self {
            MatchPattern::Literal(lit) => {Ok(ir::MatchCond::Literal(lit.translate(table)?))},
            MatchPattern::Variable(_id) => todo!("not implemented"),//desgin decision
            MatchPattern::Wildcard => Ok(ir::MatchCond::Any),
        }
    }
}


impl<'ctx> Translate<'ctx,ir::Block<'ctx>>  for MatchOut {
    fn translate(self, table: &StringTable<'ctx>) -> Result<ir::Block<'ctx>, ErrList> {
        match self {
            MatchOut::Value(val) => Ok(ir::Block::new_simple(val.translate(table)?)),
            MatchOut::Block(block) => block.translate(table),
        }
    }
}

impl<'ctx> Translate<'ctx,ir::Value<'ctx>>  for Literal {
    fn translate(self, _table: &StringTable<'ctx>) -> Result<ir::Value<'ctx>, ErrList> {
        match self {
            Literal::Int(i) => Ok(ir::Value::Int(i)),
            Literal::Float(f) => Ok(ir::Value::Float(f)),
            Literal::Atom(a) => Ok(ir::Value::Atom(a)),
            Literal::String(s) => Ok(ir::Value::String(GcPointer::new(_table.get_string(s).unwrap().to_string()))),
            Literal::Bool(b) => Ok(ir::Value::Bool(b)),
            Literal::Nil => Ok(ir::Value::Nil),
        }
    }
}
