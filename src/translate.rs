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
        let func_body = func_dec.body.translate(table);
        
        // Add function to global scope
        self.add(func_name_id, func_body, func_sig)?;
        Ok(())
    }

    fn add_import(&mut self, _import_func: ImportFunc, _table: &StringTable<'ctx>) -> Result<(), ErrList> {
        todo!("Handling of imports is not yet implemented.");
    }
}


pub trait Translate<'ctx, Output> {
    fn translate(self, table: &StringTable<'ctx>) -> Output;
}


impl<'ctx> Translate<'ctx,LazyVal<'ctx>> for FValue {
	fn translate(self, table: &StringTable<'ctx>) -> LazyVal<'ctx> {
		let val : Value = self.into();
		val.translate(table)
	}
}

impl<'ctx> Translate<'ctx,LazyVal<'ctx>> for Value {
	fn translate(self, table: &StringTable<'ctx>) -> LazyVal<'ctx> {
		match self {
			Value::Variable(id) => LazyVal::Ref(id),
			Value::Int(x) => LazyVal::Terminal(ir::Value::Int(x)),
			Value::Float(x) => LazyVal::Terminal(ir::Value::Float(x)),
			Value::Bool(x) => LazyVal::Terminal(ir::Value::Bool(x)),
			Value::Atom(x) => LazyVal::Terminal(ir::Value::Atom(x)),
			
			Value::Nil => LazyVal::Terminal(ir::Value::Nil),
			Value::FuncCall(call) =>LazyVal::FuncCall(call.translate(table)),


			Value::String(x) => {
				let r = table.get_string(x).unwrap();
				let s = r[1..r.len()-1].to_string();

				LazyVal::Terminal(ir::Value::String(
					GcPointer::new(s)
				))
			},
			


			Value::BuildIn(op) => LazyVal::Terminal(
				ir::Value::Func(
				ir::FunctionHandle::FFI(
					get_buildin_function(op)
				)
			)),

			Value::Lambda(f) => LazyVal::MakeFunc(f.translate(table)),
			Value::MatchLambda(f) => LazyVal::MakeMatchFunc(f.translate(table)),

			Value::Match(m) => {
				let (var_in,statment) = m.translate(table);
				let var = Box::new(var_in);
				LazyVal::Match{var,statment}
			},
		}
	}
}

impl<'ctx> Translate<'ctx,ir::Call<'ctx>>  for FunctionCall {
	fn translate(self, table: &StringTable<'ctx>) -> ir::Call<'ctx> {
		ir::Call{
			called:Box::new(self.name.translate(table)),
			args:self.args.into_iter().map(|x| x.translate(table)).collect(),
			debug_span: self.debug_span
		}
	}
}



impl<'ctx> Translate<'ctx,ir::Block<'ctx>>  for FuncBlock {
	fn translate(self, table: &StringTable<'ctx>) -> ir::Block<'ctx> {
		let mut ans = Vec::with_capacity(self.body.len()+1);
		for s in self.body.into_iter() {
			ans.push(s.translate(table));
		}
		ans.push(ir::Statment::Return (match self.ret {
			None => ir::GenericRet::Local(LazyVal::Terminal(ir::Value::Nil)),
			Some(r) => match r {
				Ret::Imp(x) => ScopeRet::Local(x.translate(table)),
				Ret::Exp(x) => ScopeRet::Unwind(x.translate(table)),
			},
		}));

		ir::Block::new(ans)
	}
}

impl<'ctx> Translate<'ctx,ir::MatchStatment<'ctx>>  for (Vec<MatchArm>,Span) {
	fn translate(self, table: &StringTable<'ctx>) -> ir::MatchStatment<'ctx> {
		let mut conds = Vec::with_capacity(self.0.len());
		let mut vals = Vec::with_capacity(self.0.len());

		for a in self.0.into_iter() {
			conds.push(a.pattern.translate(table));
			vals.push(a.result.translate(table));
		}

		ir::MatchStatment::new(conds,vals,self.1)
	}
} 

impl<'ctx> Translate<'ctx,(LazyVal<'ctx>,ir::MatchStatment<'ctx>) >  for MatchStatment {
	fn translate(self, table: &StringTable<'ctx>) -> (LazyVal<'ctx>,ir::MatchStatment<'ctx>) {
		let var = self.val.translate(table);
		let statment = (self.arms,self.debug_span).translate(table);
		(var, statment)
	}
}

impl<'ctx> Translate<'ctx,ir::LazyMatch<'ctx>>  for MatchLambda {
    fn translate(self, table: &StringTable<'ctx>) -> ir::LazyMatch<'ctx> {
        let match_statement = (self.arms, self.debug_span).translate(table);
        ir::LazyMatch::new(match_statement)
    }
}

impl<'ctx> Translate<'ctx,ir::LazyFunc<'ctx>>  for Lambda {
    fn translate(self, table: &StringTable<'ctx>) -> ir::LazyFunc<'ctx> {
        let sig = ir::FuncSig { arg_ids: self.sig };
        let body = self.body.translate(table);
        ir::LazyFunc::new(sig, body)
    }
}

impl<'ctx> Translate<'ctx,ir::Statment<'ctx>> for Statment {
	fn translate(self, table: &StringTable<'ctx>) -> ir::Statment<'ctx> {
		match self {
			Statment::Assign(id,x) => ir::Statment::Assign(id,x.translate(table)),
			Statment::Call(f) => ir::Statment::Call(f.translate(table)),
			Statment::Match(m) => ir::Statment::Match(m.translate(table)),
		}
	}
}

impl<'ctx> Translate<'ctx,ir::MatchCond<'ctx>> for MatchPattern {
    fn translate(self, table: &StringTable<'ctx>) -> ir::MatchCond<'ctx> {
        match self {
            MatchPattern::Literal(lit) => {ir::MatchCond::Literal(lit.translate(table))},
            MatchPattern::Variable(_id) => unreachable!("not implemented"),
            MatchPattern::Wildcard => ir::MatchCond::Any,
        }
    }
}


impl<'ctx> Translate<'ctx,ir::Block<'ctx>>  for MatchOut {
    fn translate(self, table: &StringTable<'ctx>) -> ir::Block<'ctx> {
        match self {
            MatchOut::Value(val) => ir::Block::new_simple(val.translate(table)),
            MatchOut::Block(block) => block.translate(table),
        }
    }
}

impl<'ctx> Translate<'ctx,ir::Value<'ctx>>  for Literal {
    fn translate(self, _table: &StringTable<'ctx>) -> ir::Value<'ctx> {
        match self {
            Literal::Int(i) => ir::Value::Int(i),
            Literal::Float(f) => ir::Value::Float(f),
            Literal::Atom(a) => ir::Value::Atom(a),
            Literal::String(s) => ir::Value::String(GcPointer::new(_table.get_string(s).unwrap().to_string())),
            Literal::Bool(b) => ir::Value::Bool(b),
            Literal::Nil => ir::Value::Nil,
        }
    }
}
