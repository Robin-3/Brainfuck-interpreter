use super::error::InterpreterError;

pub type Commands = Vec<Command>;

enum CommandClassic {
    Increase,
    Decrease,
    Left,
    Right,
    Input,
    Output,
    OpenLoop,
    ClosedLoop,
}

impl CommandClassic {
    pub fn code_to_tokens(code: String) -> Result<Vec<CommandClassic>, InterpreterError> {
        let mut tokens = Vec::with_capacity(code.len());
        for c in code.chars() {
            // Match each character to its corresponding Brainfuck command
            match c {
                '+' => tokens.push(Self::Increase),
                '-' => tokens.push(Self::Decrease),
                '<' => tokens.push(Self::Left),
                '>' => tokens.push(Self::Right),
                ',' => tokens.push(Self::Input),
                '.' => tokens.push(Self::Output),
                '[' => tokens.push(Self::OpenLoop),
                ']' => tokens.push(Self::ClosedLoop),
                char => return Err(InterpreterError::InstruccionUnknown(char)),
            }
        }

        Ok(tokens)
    }
}

#[derive(Clone, PartialEq)]
pub enum BufferOptions {
    Input,
    Output,
}

#[derive(Clone, PartialEq)]
pub enum LoopOptions {
    ResetCell,
    ToLeft,
    ToRight,
    PointerStart(Option<usize>),
    PointerEnd(Option<usize>),
}

// Enum to represent the Brainfuck language commands
#[derive(Clone, PartialEq)]
pub enum Command {
    Add(u8),
    Move(u16),
    Buffer(BufferOptions),
    Loop(LoopOptions),
}

impl Command {
    // Generate tokens from Brainfuck code
    pub fn code_to_tokens(code: String) -> Result<Commands, InterpreterError> {
        let mut tokens: Commands = CommandClassic::code_to_tokens(code)?
            .iter()
            .map(|command| match command {
                CommandClassic::Increase => Self::Add(1),
                CommandClassic::Decrease => Self::Add(u8::MAX),
                CommandClassic::Left => Self::Move(u16::MAX),
                CommandClassic::Right => Self::Move(1),
                CommandClassic::Input => Self::Buffer(BufferOptions::Input),
                CommandClassic::Output => Self::Buffer(BufferOptions::Output),
                CommandClassic::OpenLoop => Self::Loop(LoopOptions::PointerStart(None)),
                CommandClassic::ClosedLoop => Self::Loop(LoopOptions::PointerEnd(None)),
            })
            .collect();

        tokens = Self::add_advanced_tokens(&tokens.clone());

        // Return the generated tokens
        Self::loop_conection(&tokens.clone())
    }

    fn add_advanced_tokens(commands: &Commands) -> Commands {
        let mut tokens: Commands = Vec::with_capacity(commands.capacity());
        let mut index = 0usize;

        loop {
            let token = commands.get(index);

            match token {
                Some(command) => match command {
                    Self::Add(_) => {
                        let (value, new_index) = Self::add_token(commands, index);
                        tokens.push(Self::Add(value));
                        index = new_index;
                        continue;
                    }
                    Self::Move(_) => {
                        let (value, new_index) = Self::move_token(commands, index);
                        tokens.push(Self::Move(value));
                        index = new_index;
                        continue;
                    }
                    Self::Loop(LoopOptions::PointerStart(_)) => {
                        let (value, new_index) = Self::loop_token(commands, index);
                        tokens.push(value);
                        index = new_index;
                        continue;
                    }
                    Self::Loop(LoopOptions::PointerEnd(_)) => {
                        tokens.push(Self::Loop(LoopOptions::PointerEnd(None)))
                    }
                    command => tokens.push(command.clone()),
                },
                None => break,
            }

            index += 1;
        }

        tokens
    }

    fn add_token(commands: &Commands, start: usize) -> (u8, usize) {
        let mut counter = 0u8;
        let mut end = start;

        while let Some(Self::Add(value)) = commands.get(end) {
            counter = counter.wrapping_add(*value);
            end += 1;
        }

        (counter, end)
    }

    fn move_token(commands: &Commands, start: usize) -> (u16, usize) {
        let mut counter = 0u16;
        let mut end = start;

        while let Some(Self::Move(pointer)) = commands.get(end) {
            counter = counter.wrapping_add(*pointer);
            end += 1;
        }

        (counter, end)
    }

    fn loop_token(commands: &Commands, start: usize) -> (Self, usize) {
        match commands.get(start + 2) {
            Some(Self::Loop(LoopOptions::PointerEnd(_))) => match commands.get(start + 1) {
                Some(Self::Add(1)) | Some(Self::Add(u8::MAX)) => {
                    (Self::Loop(LoopOptions::ResetCell), start + 3)
                }
                Some(Self::Move(1)) => (Self::Loop(LoopOptions::ToRight), start + 3),
                Some(Self::Move(u16::MAX)) => (Self::Loop(LoopOptions::ToLeft), start + 3),
                _ => (Self::Loop(LoopOptions::PointerStart(None)), start + 1),
            },
            _ => (Self::Loop(LoopOptions::PointerStart(None)), start + 1),
        }
    }

    fn loop_conection(commands: &Commands) -> Result<Commands, InterpreterError> {
        let mut open_loop: Vec<usize> = Vec::with_capacity(Self::token_counter(
            commands,
            Self::Loop(LoopOptions::PointerStart(None)),
        ));
        let mut loops: Vec<(usize, usize)> = Vec::with_capacity(open_loop.capacity());

        for (index, token) in commands.iter().enumerate() {
            match token {
                Self::Loop(LoopOptions::PointerStart(_)) => open_loop.push(index),
                Self::Loop(LoopOptions::PointerEnd(_)) => match open_loop.pop() {
                    Some(open_index) => loops.push((open_index, index)),
                    None => return Err(InterpreterError::MalformedClosedLoop(index + 1)),
                },
                _ => continue,
            }
        }

        if !open_loop.is_empty() {
            return Err(InterpreterError::MalformedOpenLoop(open_loop[0] + 1));
        }

        let mut commands = commands.clone();
        while let Some((open_loop, closed_loop)) = loops.pop() {
            commands[open_loop] = Self::Loop(LoopOptions::PointerStart(Some(closed_loop)));
            commands[closed_loop] = Self::Loop(LoopOptions::PointerEnd(Some(open_loop)));
        }

        Ok(commands)
    }

    pub fn token_counter(commands: &Commands, token: Self) -> usize {
        let mut counter = 0usize;
        for c in commands.iter() {
            if token == *c {
                counter += 1
            }
        }

        counter
    }
}
