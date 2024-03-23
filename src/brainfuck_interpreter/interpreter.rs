use super::{
    command::{BufferOptions, Command, Commands, LoopOptions},
    error::InterpreterError,
};
use std::cell::OnceCell;

pub type Data = Vec<u8>;

// Struct to represent the Brainfuck interpreter
#[derive(Default)]
pub struct Interpreter {
    args: OnceCell<Data>,
    output: OnceCell<Data>,
    tokens: OnceCell<Commands>,
}

impl Interpreter {
    // Constructor to create a new Brainfuck interpreter instance
    pub fn new() -> Self {
        Self::default()
    }

    fn set_args(&mut self, args: Data) -> Result<(), InterpreterError> {
        self.args
            .set(args)
            .map_err(|_| InterpreterError::ArgsOverwritten)?;

        Ok(())
    }

    fn set_tokens(&self, tokens: Commands) -> Result<(), InterpreterError> {
        self.tokens
            .set(tokens)
            .map_err(|_| InterpreterError::TokensOverwritten)?;

        Ok(())
    }

    // Execute the Brainfuck code
    fn run_code(&mut self) -> Result<(), InterpreterError> {
        if self.output.get().is_some() {
            return Ok(());
        }

        match self.tokens.get() {
            Some(tokens) => {
                let mut memory = [0u8; u16::MAX as usize + 1];
                let mut memory_pointer = 0usize;
                let mut output: Data =
                    // It is an initial value; the true one is unknown because it could be within a loop, hence it could be greater (if it repeats any loop) or smaller (if it didn't enter any loop).
                    Vec::with_capacity(Command::token_counter(tokens, Command::Buffer(BufferOptions::Output)));
                let mut token_index = 0usize;
                let mut args: Option<Data> = self.args.get().cloned();

                while let Some(token) = tokens.get(token_index) {
                    // Match each command and perform the corresponding operation
                    match token {
                        Command::Add(increment) => {
                            memory[memory_pointer] = memory[memory_pointer].wrapping_add(*increment)
                        }
                        Command::Move(pointer) => {
                            memory_pointer = (memory_pointer as u16).wrapping_add(*pointer) as usize
                        }
                        Command::Buffer(BufferOptions::Input) => match args.as_mut() {
                            Some(bf_args) => match bf_args.pop() {
                                Some(value) => memory[memory_pointer] = value,
                                None => {
                                    memory[memory_pointer] = 0;
                                    args = None;
                                } // EOF
                            },
                            None => return Err(InterpreterError::MissingArgs),
                        },
                        Command::Buffer(BufferOptions::Output) => {
                            output.push(memory[memory_pointer])
                        }
                        Command::Loop(LoopOptions::PointerStart(None), _)
                        | Command::Loop(LoopOptions::PointerEnd(None), _) => {
                            return Err(InterpreterError::UnconnectedLoops)
                        }
                        Command::Loop(LoopOptions::Comment, index_file) => {
                            if memory[memory_pointer] != 0 {
                                return Err(InterpreterError::InfinityLoopFound(
                                    *index_file,
                                    memory[memory_pointer],
                                    memory_pointer,
                                ));
                            }
                        }
                        Command::Loop(LoopOptions::AddToReset(add_is_even), index_file) => {
                            let current_is_even = memory[memory_pointer] % 2 == 0;

                            match (*add_is_even, current_is_even) {
                                (true, true) => memory[memory_pointer] = 0,
                                (false, _) => memory[memory_pointer] = 0,
                                _ => {
                                    return Err(InterpreterError::InfinityLoopFound(
                                        *index_file,
                                        memory[memory_pointer],
                                        memory_pointer,
                                    ))
                                }
                            }

                            memory[memory_pointer] = 0;
                        }
                        Command::Loop(LoopOptions::ToRight, _) => loop {
                            if memory[memory_pointer] == 0 {
                                break;
                            }
                            memory_pointer = (memory_pointer as u16).wrapping_add(1) as usize;
                        },
                        Command::Loop(LoopOptions::ToLeft, _) => loop {
                            if memory[memory_pointer] == 0 {
                                break;
                            }
                            memory_pointer = (memory_pointer as u16).wrapping_sub(1) as usize;
                        },
                        Command::Loop(LoopOptions::PointerStart(Some(pointer)), _) => {
                            if memory[memory_pointer] == 0 {
                                token_index = *pointer;
                            }
                        }
                        Command::Loop(LoopOptions::PointerEnd(Some(pointer)), _) => {
                            if memory[memory_pointer] != 0 {
                                token_index = *pointer;
                            }
                        }
                    }

                    token_index += 1;
                }

                self.output
                    .set(output)
                    .map_err(|_| InterpreterError::OutputOverwritten)?;
            }
            None => return Err(InterpreterError::TokensUnknown),
        }

        Ok(())
    }

    pub fn execute(
        &mut self,
        tokens: Commands,
        args: Option<Data>,
    ) -> Result<(), InterpreterError> {
        if let Some(bf_args) = args {
            self.set_args(bf_args)?;
        }
        self.set_tokens(tokens)?;
        self.run_code()
    }

    pub fn get_output_as_vec(&self) -> Result<Data, InterpreterError> {
        match self.output.get() {
            Some(output) => Ok(output.clone()),
            None => Err(InterpreterError::OutputUnknown),
        }
    }

    pub fn get_output_as_string(&self) -> Result<String, InterpreterError> {
        match self.output.get() {
            Some(output) => Ok(String::from_utf8_lossy(output.as_slice()).to_string()),
            None => Err(InterpreterError::OutputUnknown),
        }
    }
}
