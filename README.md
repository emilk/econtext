# econtext: fast and simple error context on panics.

Calling an `econtext!` macro adds a scope to a thread-local linked list. If there is a `panic!()`
while the scope is active, the data/message provided to the `econtext!` macro will be printed.

This thus provides an opt-in stack trace with optional data (e.g. the values of function arguments).

This can be very useful, for instance:

* To print what data was being worked on when an error occurred
* To provide something similar to a stack trace where a real stack trace is not available (e.g. in some WASM contexts)
* To print a shorter and more readable stack trace for when the real stack trace is too long and winding.

The overhead of calling an `excontext` macro is around 15ns on a 2020 MacBook Pro.

## Example
``` rust
use econtext::*;

fn main() {
	econtext::add_panic_hook(); // Ensures econtext is printed on panic
	econtext!("While running"); // Print a message if there is a panic
	run();
}

fn run() {
	econtext_function!(); // Print function name (`run`) if there is a panic
	process("filename.txt");
}

fn process(filename: &str) {
	econtext_function_data!(filename.to_owned()); // Print function name and filename if there is a panic
	for i in 0..10 {
		econtext_data!("i", i); // Print loop index if there is a panic
		assert!(i != 4, "Intentional panic");
	}
}
```

On error, something like this is printed:

``` text
ERROR CONTEXT:
  my_module src/main.rs:17: i 4
  my_module src/main.rs:15: main::process "filename.txt"
  my_module src/main.rs:10: main::run
  my_module src/main.rs:5: While running
```

