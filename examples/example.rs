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
