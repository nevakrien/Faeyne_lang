# Faeyne_lang
an attempt at making a functional languge heavily inspired by elixir but with monads


Faeyne stands for: 
"Functions are everything you need EXCEPTION"


The core idea is we only need piecewise functions for achiving basically everything.
Type Exceptions are also present to make things more argonomics
IO monad (and others) are implemented as follows

```
def main(system) {
	input = system(:input)
	x = pure_func(input)
	io_func(x,system(:printer))
}
```

this allows u to pass a diffrent function. for instance if you want to supress printing you can do


```io_func(x,fn(str) -> {})```