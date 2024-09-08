// auto-generated: "lalrpop 0.21.0"
// sha3: 1e92e5bd655c39ae610d312453c4ca0511f40eb10d8cafa30d9c484dde850feb
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
#[allow(unused_extern_crates)]
extern crate alloc;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::never_loop, clippy::match_single_binding, clippy::needless_raw_string_hashes)]
mod __parse__StringList {

    #[allow(unused_extern_crates)]
    extern crate lalrpop_util as __lalrpop_util;
    #[allow(unused_imports)]
    use self::__lalrpop_util::state_machine as __state_machine;
    #[allow(unused_extern_crates)]
    extern crate alloc;
    use super::__ToTriple;
    #[allow(dead_code)]
    pub(crate) enum __Symbol<'input>
     {
        Variant0(&'input str),
        Variant1(alloc::vec::Vec<&'input str>),
        Variant2(Vec<String>),
    }
    const __ACTION: &[i8] = &[
        // State 0
        0, 0, 2, 0, 0,
        // State 1
        0, 0, 5, 0, 0,
        // State 2
        0, 0, 0, 0, 0,
        // State 3
        0, 0, 6, 0, 0,
        // State 4
        0, 0, -1, 0, 0,
        // State 5
        0, 0, -2, 0, 0,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 5 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        0,
        // State 1
        -4,
        // State 2
        -5,
        // State 3
        -3,
        // State 4
        -1,
        // State 5
        -2,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            0 => 3,
            1 => 2,
            _ => 0,
        }
    }
    const __TERMINAL: &[&str] = &[
        r###""word""###,
        r###""atom""###,
        r###""string""###,
        r###""pstr""###,
        r###""UK""###,
    ];
    fn __expected_tokens(__state: i8) -> alloc::vec::Vec<alloc::string::String> {
        __TERMINAL.iter().enumerate().filter_map(|(index, terminal)| {
            let next_state = __action(__state, index);
            if next_state == 0 {
                None
            } else {
                Some(alloc::string::ToString::to_string(terminal))
            }
        }).collect()
    }
    fn __expected_tokens_from_states<
        'input,
    >(
        __states: &[i8],
        _: core::marker::PhantomData<(&'input ())>,
    ) -> alloc::vec::Vec<alloc::string::String>
    {
        __TERMINAL.iter().enumerate().filter_map(|(index, terminal)| {
            if __accepts(None, __states, Some(index), core::marker::PhantomData::<(&())>) {
                Some(alloc::string::ToString::to_string(terminal))
            } else {
                None
            }
        }).collect()
    }
    struct __StateMachine<'input>
    where 
    {
        input: &'input str,
        __phantom: core::marker::PhantomData<(&'input ())>,
    }
    impl<'input> __state_machine::ParserDefinition for __StateMachine<'input>
    where 
    {
        type Location = ();
        type Error = ();
        type Token = LexToken<'input>;
        type TokenIndex = usize;
        type Symbol = __Symbol<'input>;
        type Success = Vec<String>;
        type StateIndex = i8;
        type Action = i8;
        type ReduceIndex = i8;
        type NonterminalIndex = usize;

        #[inline]
        fn start_location(&self) -> Self::Location {
              Default::default()
        }

        #[inline]
        fn start_state(&self) -> Self::StateIndex {
              0
        }

        #[inline]
        fn token_to_index(&self, token: &Self::Token) -> Option<usize> {
            __token_to_integer(token, core::marker::PhantomData::<(&())>)
        }

        #[inline]
        fn action(&self, state: i8, integer: usize) -> i8 {
            __action(state, integer)
        }

        #[inline]
        fn error_action(&self, state: i8) -> i8 {
            __action(state, 5 - 1)
        }

        #[inline]
        fn eof_action(&self, state: i8) -> i8 {
            __EOF_ACTION[state as usize]
        }

        #[inline]
        fn goto(&self, state: i8, nt: usize) -> i8 {
            __goto(state, nt)
        }

        fn token_to_symbol(&self, token_index: usize, token: Self::Token) -> Self::Symbol {
            __token_to_symbol(token_index, token, core::marker::PhantomData::<(&())>)
        }

        fn expected_tokens(&self, state: i8) -> alloc::vec::Vec<alloc::string::String> {
            __expected_tokens(state)
        }

        fn expected_tokens_from_states(&self, states: &[i8]) -> alloc::vec::Vec<alloc::string::String> {
            __expected_tokens_from_states(states, core::marker::PhantomData::<(&())>)
        }

        #[inline]
        fn uses_error_recovery(&self) -> bool {
            false
        }

        #[inline]
        fn error_recovery_symbol(
            &self,
            recovery: __state_machine::ErrorRecovery<Self>,
        ) -> Self::Symbol {
            panic!("error recovery not enabled for this grammar")
        }

        fn reduce(
            &mut self,
            action: i8,
            start_location: Option<&Self::Location>,
            states: &mut alloc::vec::Vec<i8>,
            symbols: &mut alloc::vec::Vec<__state_machine::SymbolTriple<Self>>,
        ) -> Option<__state_machine::ParseResult<Self>> {
            __reduce(
                self.input,
                action,
                start_location,
                states,
                symbols,
                core::marker::PhantomData::<(&())>,
            )
        }

        fn simulate_reduce(&self, action: i8) -> __state_machine::SimulatedReduce<Self> {
            __simulate_reduce(action, core::marker::PhantomData::<(&())>)
        }
    }
    fn __token_to_integer<
        'input,
    >(
        __token: &LexToken<'input>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> Option<usize>
    {
        match __token {
            LexToken::Word(_) if true => Some(0),
            LexToken::Atom(_) if true => Some(1),
            LexToken::String(_) if true => Some(2),
            LexToken::PoisonString(_) if true => Some(3),
            LexToken::Unknowen(_) if true => Some(4),
            _ => None,
        }
    }
    fn __token_to_symbol<
        'input,
    >(
        __token_index: usize,
        __token: LexToken<'input>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> __Symbol<'input>
    {
        #[allow(clippy::manual_range_patterns)]match __token_index {
            0 | 1 | 2 | 3 | 4 => match __token {
                LexToken::Word(__tok0) | LexToken::Atom(__tok0) | LexToken::String(__tok0) | LexToken::PoisonString(__tok0) | LexToken::Unknowen(__tok0) if true => __Symbol::Variant0(__tok0),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
    fn __simulate_reduce<
        'input,
    >(
        __reduce_index: i8,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> __state_machine::SimulatedReduce<__StateMachine<'input>>
    {
        match __reduce_index {
            0 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 0,
                }
            }
            1 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 0,
                }
            }
            2 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 1,
                }
            }
            3 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 1,
                }
            }
            4 => __state_machine::SimulatedReduce::Accept,
            _ => panic!("invalid reduction index {}", __reduce_index)
        }
    }
    pub struct StringListParser {
        _priv: (),
    }

    impl Default for StringListParser { fn default() -> Self { Self::new() } }
    impl StringListParser {
        pub fn new() -> StringListParser {
            StringListParser {
                _priv: (),
            }
        }

        #[allow(dead_code)]
        pub fn parse<
            'input,
            __TOKEN: __ToTriple<'input, >,
            __TOKENS: IntoIterator<Item=__TOKEN>,
        >(
            &self,
            input: &'input str,
            __tokens0: __TOKENS,
        ) -> Result<Vec<String>, __lalrpop_util::ParseError<(), LexToken<'input>, ()>>
        {
            let __tokens = __tokens0.into_iter();
            let mut __tokens = __tokens.map(|t| __ToTriple::to_triple(t));
            __state_machine::Parser::drive(
                __StateMachine {
                    input,
                    __phantom: core::marker::PhantomData::<(&())>,
                },
                __tokens,
            )
        }
    }
    fn __accepts<
        'input,
    >(
        __error_state: Option<i8>,
        __states: &[i8],
        __opt_integer: Option<usize>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> bool
    {
        let mut __states = __states.to_vec();
        __states.extend(__error_state);
        loop {
            let mut __states_len = __states.len();
            let __top = __states[__states_len - 1];
            let __action = match __opt_integer {
                None => __EOF_ACTION[__top as usize],
                Some(__integer) => __action(__top, __integer),
            };
            if __action == 0 { return false; }
            if __action > 0 { return true; }
            let (__to_pop, __nt) = match __simulate_reduce(-(__action + 1), core::marker::PhantomData::<(&())>) {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop, nonterminal_produced
                } => (states_to_pop, nonterminal_produced),
                __state_machine::SimulatedReduce::Accept => return true,
            };
            __states_len -= __to_pop;
            __states.truncate(__states_len);
            let __top = __states[__states_len - 1];
            let __next_state = __goto(__top, __nt);
            __states.push(__next_state);
        }
    }
    fn __reduce<
        'input,
    >(
        input: &'input str,
        __action: i8,
        __lookahead_start: Option<&()>,
        __states: &mut alloc::vec::Vec<i8>,
        __symbols: &mut alloc::vec::Vec<((),__Symbol<'input>,())>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> Option<Result<Vec<String>,__lalrpop_util::ParseError<(), LexToken<'input>, ()>>>
    {
        let (__pop_states, __nonterminal) = match __action {
            0 => {
                __reduce0(input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            1 => {
                __reduce1(input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            2 => {
                __reduce2(input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            3 => {
                __reduce3(input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            4 => {
                // __StringList = StringList => ActionFn(0);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0;
                let __end = __sym0.2;
                let __nt = super::__action0::<>(input, __sym0);
                return Some(Ok(__nt));
            }
            _ => panic!("invalid action code {}", __action)
        };
        let __states_len = __states.len();
        __states.truncate(__states_len - __pop_states);
        let __state = *__states.last().unwrap();
        let __next_state = __goto(__state, __nonterminal);
        __states.push(__next_state);
        None
    }
    #[inline(never)]
    fn __symbol_type_mismatch() -> ! {
        panic!("symbol type mismatch")
    }
    fn __pop_Variant2<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<((),__Symbol<'input>,())>
    ) -> ((), Vec<String>, ())
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<((),__Symbol<'input>,())>
    ) -> ((), alloc::vec::Vec<&'input str>, ())
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant0<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<((),__Symbol<'input>,())>
    ) -> ((), &'input str, ())
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant0(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __reduce0<
        'input,
    >(
        input: &'input str,
        __lookahead_start: Option<&()>,
        __symbols: &mut alloc::vec::Vec<((),__Symbol<'input>,())>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // "string"+ = "string" => ActionFn(3);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0;
        let __end = __sym0.2;
        let __nt = super::__action3::<>(input, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 0)
    }
    fn __reduce1<
        'input,
    >(
        input: &'input str,
        __lookahead_start: Option<&()>,
        __symbols: &mut alloc::vec::Vec<((),__Symbol<'input>,())>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // "string"+ = "string"+, "string" => ActionFn(4);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0;
        let __end = __sym1.2;
        let __nt = super::__action4::<>(input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (2, 0)
    }
    fn __reduce2<
        'input,
    >(
        input: &'input str,
        __lookahead_start: Option<&()>,
        __symbols: &mut alloc::vec::Vec<((),__Symbol<'input>,())>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // StringList = "string", "string"+ => ActionFn(1);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0;
        let __end = __sym1.2;
        let __nt = super::__action1::<>(input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (2, 1)
    }
    fn __reduce3<
        'input,
    >(
        input: &'input str,
        __lookahead_start: Option<&()>,
        __symbols: &mut alloc::vec::Vec<((),__Symbol<'input>,())>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // StringList = "string" => ActionFn(2);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0;
        let __end = __sym0.2;
        let __nt = super::__action2::<>(input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 1)
    }
}
#[allow(unused_imports)]
pub use self::__parse__StringList::StringListParser;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits)]
fn __action0<
    'input,
>(
    input: &'input str,
    (_, __0, _): ((), Vec<String>, ()),
) -> Vec<String>
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits)]
fn __action1<
    'input,
>(
    input: &'input str,
    (_, first, _): ((), &'input str, ()),
    (_, rest, _): ((), alloc::vec::Vec<&'input str>, ()),
) -> Vec<String>
{
    {
        let mut strings = vec![first.to_string()];
        strings.extend(rest.into_iter().map(|s| s.to_string()));
        strings
    }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits)]
fn __action2<
    'input,
>(
    input: &'input str,
    (_, single, _): ((), &'input str, ()),
) -> Vec<String>
{
    vec![single.to_string()]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits)]
fn __action3<
    'input,
>(
    input: &'input str,
    (_, __0, _): ((), &'input str, ()),
) -> alloc::vec::Vec<&'input str>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits)]
fn __action4<
    'input,
>(
    input: &'input str,
    (_, v, _): ((), alloc::vec::Vec<&'input str>, ()),
    (_, e, _): ((), &'input str, ()),
) -> alloc::vec::Vec<&'input str>
{
    { let mut v = v; v.push(e); v }
}
#[allow(clippy::type_complexity, dead_code)]

pub  trait __ToTriple<'input, >
{
    fn to_triple(value: Self) -> Result<((),LexToken<'input>,()), __lalrpop_util::ParseError<(), LexToken<'input>, ()>>;
}

impl<'input, > __ToTriple<'input, > for ((), LexToken<'input>, ())
{
    fn to_triple(value: Self) -> Result<((),LexToken<'input>,()), __lalrpop_util::ParseError<(), LexToken<'input>, ()>> {
        Ok(value)
    }
}
impl<'input, > __ToTriple<'input, > for Result<((), LexToken<'input>, ()), ()>
{
    fn to_triple(value: Self) -> Result<((),LexToken<'input>,()), __lalrpop_util::ParseError<(), LexToken<'input>, ()>> {
        match value {
            Ok(v) => Ok(v),
            Err(error) => Err(__lalrpop_util::ParseError::User { error }),
        }
    }
}
