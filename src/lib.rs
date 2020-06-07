//! # econtext: fast and simple error context on panics.
//!
//! Calling an `econtext!` macro adds a scope to a thread-local linked list. If there is a `panic!()`
//! while the scope is active, the data/message provided to the `econtext!` macro will be printed.
//!
//! This thus provides an opt-in stack trace with optional data (e.g. the values of function arguments).
//!
//! This can be very useful, for instance:
//!
//! * To print what data was being worked on when an error occurred
//! * To provide something similar to a stack trace where a real stack trace is not available (e.g. in some WASM contexts)
//! * To print a shorter and more readable stack trace for when the real stack trace is too long and winding.
//!
//! The overhead of calling an `excontext` macro is around 15ns on a 2020 MacBook Pro.
//!
//! ## Example
//! ``` rust
//! use econtext::*;
//!
//! fn main() {
//! 	econtext::add_panic_hook(); // Ensures econtext is printed on panic
//! 	econtext!("While running"); // Print a message if there is a panic
//! 	run();
//! }
//!
//! fn run() {
//! 	econtext_function!(); // Print function name (`run`) if there is a panic
//! 	process("filename.txt");
//! }
//!
//! fn process(filename: &str) {
//! 	econtext_function_data!(filename.to_owned()); // Print function name and filename if there is a panic
//! 	for i in 0..10 {
//! 		econtext_data!("i", i); // Print loop index if there is a panic
//! 		assert!(i != 4, "Intentional panic");
//! 	}
//! }
//! ```
//!
//! On error, something like this is printed:
//!
//! ``` text
//! ERROR CONTEXT:
//!   my_module src/main.rs:17: i 4
//!   my_module src/main.rs:15: main::process "filename.txt"
//!   my_module src/main.rs:10: main::run
//!   my_module src/main.rs:5: While running
//! ```

use std::{cell::RefCell, fmt::Debug};

// Points to the top of the error context stack
thread_local! {
	pub static ERROR_STACK: RefCell<Option<*const dyn Entry>> = RefCell::new(None);
}

/// The trait for an entry in the stack
pub trait Entry {
	fn write(&self, writer: &mut dyn std::fmt::Write);
}

// ----------------------------------------------------------------------------

/// What is put in a stack frame that uses the macros.
pub struct DataScope<Data> {
	/// Linked list: pointer to the previous entry.
	previous: Option<*const dyn Entry>,

	module_path: &'static str,
	file: &'static str,
	line: u32,

	message: &'static str,
	data: Data,
}

impl<Data: Debug> Entry for DataScope<Data> {
	fn write(&self, writer: &mut dyn std::fmt::Write) {
		write!(
			writer,
			"  {} {}:{}: {} {:?}\n",
			self.module_path, self.file, self.line, self.message, self.data
		)
		.ok();
		unsafe {
			if let Some(previous) = self.previous.as_ref().and_then(|p| p.as_ref()) {
				previous.write(writer);
			}
		}
	}
}

impl<Data: Debug> DataScope<Data> {
	pub fn new(module_path: &'static str, file: &'static str, line: u32, message: &'static str, data: Data) -> Self {
		let previous = ERROR_STACK.with(|stack| stack.borrow().clone());
		DataScope {
			previous,
			module_path,
			file,
			line,
			message,
			data,
		}
	}
}

impl<Data> Drop for DataScope<Data> {
	fn drop(&mut self) {
		ERROR_STACK.with(|stack| *stack.borrow_mut() = self.previous);
	}
}

// ----------------------------------------------------------------------------

/// Used internally when not having any data in a context scope.
pub struct EmptyDebug {}
impl std::fmt::Debug for EmptyDebug {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Ok(())
	}
}

// ----------------------------------------------------------------------------

/// Prints all active error contexts to stderr.
///
/// Example printout:
///
/// ``` text
/// ERROR CONTEXT:
///   example examples/example.rs:7: i 4
///   example examples/example.rs:4: example::process_file "file.txt"
///   example examples/example.rs:13: example::do_stuff
///   example examples/example.rs:20: main()
/// ```
pub fn print_econtext() {
	let context = econtext_string();
	if !context.is_empty() {
		eprintln!("ERROR CONTEXT:");
		eprintln!("{}", context);
	}
}

/// Returns the error context as a string.
///
/// ``` text
///   example examples/example.rs:7: i 4
///   example examples/example.rs:4: example::process_file "file.txt"
///   example examples/example.rs:13: example::do_stuff
///   example examples/example.rs:20: main()
/// ```
pub fn econtext_string() -> String {
	ERROR_STACK.with(|value| unsafe {
		if let Some(entry) = value.borrow().as_ref().and_then(|p| p.as_ref()) {
			let mut output = String::new();
			entry.write(&mut output);
			output
		} else {
			Default::default()
		}
	})
}

/// Call this once to add a panic hook that calls `print_econtext()`.
pub fn add_panic_hook() {
	let previous_hook = std::panic::take_hook();

	std::panic::set_hook(Box::new(move |panic_info: &std::panic::PanicInfo| {
		print_econtext();
		previous_hook(panic_info);
	}));
}

// ----------------------------------------------------------------------------

pub fn type_name_of<T>(_: T) -> &'static str {
	std::any::type_name::<T>()
}

#[macro_export]
macro_rules! current_function_name {
	() => {{
		fn f() {}
		let name = $crate::type_name_of(f);
		// Remove "::f" from the name:
		&name.get(..name.len() - 3).unwrap()
		}};
}

// ----------------------------------------------------------------------------

/// Provide a single `&'static str` message as context.
///
/// Example: `econtext!("cleaning the floor");'
///
/// This has a very low overhead of around 15 ns on a 2020 MacBook Pro.
#[macro_export]
macro_rules! econtext {
	($message:expr) => {
		let _scope = $crate::DataScope::new(module_path!(), file!(), line!(), $message, $crate::EmptyDebug {});
		$crate::ERROR_STACK.with(|stack| *stack.borrow_mut() = Some(&_scope));
	};
}

/// Provide a `&'static str` and some data as context.
///
/// Example: `econtext_data!("loop index", i);'
///
/// This has a very low overhead of around 15 ns on a 2020 MacBook Pro.
///
/// Unfortunately `econtext_data!` does not support references, so things like &str must be converted into their owned versions,
/// e.g. `econtext_data!("file_name", file_name.to_owned());'.
#[macro_export]
macro_rules! econtext_data {
	($message:expr, $data:expr) => {
		let _scope = $crate::DataScope::new(module_path!(), file!(), line!(), $message, $data);
		$crate::ERROR_STACK.with(|stack| *stack.borrow_mut() = Some(&_scope));
	};
}

/// Provide current function name as context.
///
/// Example: `econtext_function!();'
///
/// This has a very low overhead of around 15 ns on a 2020 MacBook Pro.
#[macro_export]
macro_rules! econtext_function {
	() => {
		let _scope = $crate::DataScope::new(
			module_path!(),
			file!(),
			line!(),
			$crate::current_function_name!(),
			$crate::EmptyDebug {},
			);
		$crate::ERROR_STACK.with(|stack| *stack.borrow_mut() = Some(&_scope));
	};
}

/// Provide current function name and some data as context.
///
/// Example: `econtext_function_data!(function_argument);'
///
/// This has a very low overhead of around 15 ns on a 2020 MacBook Pro.
///
/// Unfortunately `econtext_function_data!` does not support references, so things like &str must be converted into their owned versions,
/// e.g. `econtext_function_data!("file_name", file_name.to_owned());'.
#[macro_export]
macro_rules! econtext_function_data {
	($data:expr) => {
		let _scope = $crate::DataScope::new(
			module_path!(),
			file!(),
			line!(),
			$crate::current_function_name!(),
			$data,
			);
		$crate::ERROR_STACK.with(|stack| *stack.borrow_mut() = Some(&_scope));
	};
}
