use thiserror::Error;

// Define custom error types using the `thiserror` crate
#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("Usage: {0}")]
    SintaxisError(String),
    #[error("Character instruction unknown: '{0}'")]
    InstruccionUnknown(char),
    #[error("Cannot parse argument `{0}`: {1}")]
    ParseError(String, #[source] std::num::ParseIntError),
    #[error("Closed loop does not match an open loop at index: {0}")]
    MalformedClosedLoop(usize),
    #[error("Open loop does not match a closed loop at index: {0}")]
    MalformedOpenLoop(usize),
    #[error("Missing arguments")]
    MissingArgs,
    #[error("Execution Error: Tokens not loaded")]
    TokensUnknown,
    #[error("Execution Error: Tokens cannot be overwritten")]
    TokensOverwritten,
    #[error("Execution Error: Args cannot be overwritten")]
    ArgsOverwritten,
    #[error("Execution Error: No code was executed")]
    OutputUnknown,
    #[error("Execution Error: Unconnected loops")]
    UnconnectedLoops,
    #[error("Unexpected Error: Modifying output value")]
    OutputOverwritten,
}
