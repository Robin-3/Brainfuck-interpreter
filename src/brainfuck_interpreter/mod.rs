use command::Command;
use error::InterpreterError;
use interpreter::{Data, Interpreter};

mod command;
mod error;
mod interpreter;

const USAGE: &str = "./brainfuck <bf_code> [bf_args]\n\nBrainfuck interpreter.\n\nArguments:\n  <bf_code>        Brainfuck code to be executed. Use only the following 8 instructions: +-.,[]<>\n  [bf_args]        Pass a single string parameter to be converted into a collection of u8 characters (ascii).\n                   Pass a collection of u8 numbers (0 to 255).";

// Function to interpret Brainfuck code from command line arguments
pub fn brainfuck_interpreter() -> Result<(String, Data), InterpreterError> {
    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();
    let bf_args: Option<Data> = match args.len() {
        2 => None,
        3 => Some(args[2].as_bytes().iter().rev().copied().collect()),
        n if n > 3 => {
            let args = args
                .iter()
                .skip(2)
                .rev()
                .map(|arg| {
                    arg.parse()
                        .map_err(|e| InterpreterError::ParseError(arg.to_string(), e))
                })
                .collect::<Result<Data, InterpreterError>>()?;
            Some(args)
        }
        _ => return Err(InterpreterError::SintaxisError(USAGE.to_string())),
    };

    // Create a new Brainfuck instance and execute the code
    let mut bf = Interpreter::new();
    bf.execute(Command::code_to_tokens(args[1].to_string())?, bf_args)?;

    // Return the output as a tuple of String and Vec<u8>
    Ok((bf.get_output_as_string()?, bf.get_output_as_vec()?))
}
