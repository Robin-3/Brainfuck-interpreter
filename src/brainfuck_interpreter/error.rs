use thiserror::Error;

// Define custom error types using the `thiserror` crate
#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("Usage: {0}")]
    SintaxisError(String),
    #[error("Unknown character instruction `{0}` at code position `{1}`")]
    InstruccionUnknown(char, usize),
    #[error("Cannot parse argument `{0}`: {1}")]
    ParseError(String, #[source] std::num::ParseIntError),
    #[error("Closed loop does not match an open loop at index: `{0}`")]
    MalformedClosedLoop(usize),
    #[error("Open loop does not match a closed loop at index: `{0}`")]
    MalformedOpenLoop(usize),
    #[error("An infinite loop has been found, at code position `{0}`, with a current value `{1}` in memory cell `{2}`")]
    InfinityLoopFound(usize, u8, usize),
    #[error("An infinite movement loop has been found, at code position `{0}`, memory does not contain value `0`")]
    InfinityLoopMemoryFull(usize),
    #[error("An infinite movement loop has been found, at code position `{0}`, the pointer never stops moving, memory cell movement is `{1}`")]
    InfinityLoopMovement(usize, u16),
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
