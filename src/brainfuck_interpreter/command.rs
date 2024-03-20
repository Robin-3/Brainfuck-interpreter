use super::error::InterpreterError;

pub type Commands = Vec<Command>;

// Enum to represent the Brainfuck language commands
#[derive(Clone, PartialEq, Debug)]
pub enum Command {
    Increase,
    Decrease,
    Left,
    Right,
    Input,
    Output,
    OpenLoop(Option<usize>),
    ClosedLoop(Option<usize>),
    Add(u8),
    Move(u16),
    ToLeftLoop,
    ToRightLoop,
    ResetCell,
}

impl Command {
    // Generate tokens from Brainfuck code
    pub fn code_to_tokens(code: String) -> Result<Commands, InterpreterError> {
        let mut tokens: Commands = Vec::with_capacity(code.len());
        for c in code.chars() {
            // Match each character to its corresponding Brainfuck command
            match c {
                '+' => tokens.push(Self::Increase),
                '-' => tokens.push(Self::Decrease),
                '<' => tokens.push(Self::Left),
                '>' => tokens.push(Self::Right),
                ',' => tokens.push(Self::Input),
                '.' => tokens.push(Self::Output),
                '[' => tokens.push(Self::OpenLoop(None)),
                ']' => tokens.push(Self::ClosedLoop(None)),
                char => return Err(InterpreterError::InstruccionUnknown(char)),
            }
        }

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
                    Self::Increase | Self::Decrease => {
                        let (value, new_index) = Self::add_token(commands, index);
                        tokens.push(Self::Add(value));
                        index = new_index;
                        continue;
                    }
                    Self::Left | Self::Right => {
                        let (value, new_index) = Self::move_token(commands, index);
                        tokens.push(Self::Move(value));
                        index = new_index;
                        continue;
                    }
                    Self::OpenLoop(_) => match Self::loop_token(commands, index) {
                        Some(token) => {
                            tokens.push(token);
                            index += 2;
                        }
                        None => {
                            tokens.push(Self::OpenLoop(None));
                        }
                    },
                    Self::ClosedLoop(_) => tokens.push(Self::ClosedLoop(None)),
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

        loop {
            match commands.get(end) {
                Some(Self::Increase) => counter = counter.wrapping_add(1),
                Some(Self::Decrease) => counter = counter.wrapping_sub(1),
                _ => break,
            }
            end += 1;
        }

        (counter, end)
    }

    fn move_token(commands: &Commands, start: usize) -> (u16, usize) {
        let mut counter = 0u16;
        let mut end = start;

        loop {
            match commands.get(end) {
                Some(Self::Left) => counter = counter.wrapping_sub(1),
                Some(Self::Right) => counter = counter.wrapping_add(1),
                _ => break,
            }
            end += 1;
        }

        (counter, end)
    }

    fn loop_token(commands: &Commands, start: usize) -> Option<Self> {
        match commands.get(start + 2) {
            Some(Self::ClosedLoop(_)) => match commands.get(start + 1) {
                Some(Self::Increase)
                | Some(Self::Decrease)
                | Some(Self::Add(1))
                | Some(Self::Add(u8::MAX)) => Some(Self::ResetCell),
                Some(Self::Left) => Some(Self::ToLeftLoop),
                Some(Self::Right) => Some(Self::ToRightLoop),
                _ => None,
            },
            _ => None,
        }
    }

    fn loop_conection(commands: &Commands) -> Result<Commands, InterpreterError> {
        let mut open_loop: Vec<usize> =
            Vec::with_capacity(Self::token_counter(commands, Self::OpenLoop(None)));
        let mut loops: Vec<(usize, usize)> = Vec::with_capacity(open_loop.capacity());

        for (index, token) in commands.iter().enumerate() {
            match token {
                Self::OpenLoop(_) => open_loop.push(index),
                Self::ClosedLoop(_) => match open_loop.pop() {
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
            commands[open_loop] = Self::OpenLoop(Some(closed_loop));
            commands[closed_loop] = Self::ClosedLoop(Some(open_loop));
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
