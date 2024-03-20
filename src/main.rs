use brainfuck_interpreter::brainfuck_interpreter;

mod brainfuck_interpreter;

// Main function to run the brainfuck interpreter
fn main() {
    // Match the result of the Brainfuck interpreter function and print the output or error
    match brainfuck_interpreter() {
        Ok(output) => println!("\"{}\" {:?}", output.0, output.1),
        Err(error) => eprintln!("{}", error),
    }
}
