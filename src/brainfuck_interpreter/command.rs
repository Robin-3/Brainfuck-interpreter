use super::error::InterpreterError;

pub type Commands = Vec<Command>;

enum CommandClassic {
    Increase,
    Decrease,
    Left,
    Right,
    Input,
    Output,
    OpenLoop(usize),   // index_file
    ClosedLoop(usize), // index_file
}

impl CommandClassic {
    pub fn code_to_tokens(code: String) -> Result<Vec<CommandClassic>, InterpreterError> {
        let mut tokens = Vec::with_capacity(code.len());
        for (index_file, c) in code.chars().enumerate() {
            // Match each character to its corresponding Brainfuck command
            match c {
                '+' => tokens.push(Self::Increase),
                '-' => tokens.push(Self::Decrease),
                '<' => tokens.push(Self::Left),
                '>' => tokens.push(Self::Right),
                ',' => tokens.push(Self::Input),
                '.' => tokens.push(Self::Output),
                '[' => tokens.push(Self::OpenLoop(index_file + 1)),
                ']' => tokens.push(Self::ClosedLoop(index_file + 1)),
                char => return Err(InterpreterError::InstruccionUnknown(char, index_file + 1)),
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
    Comment,          // [] Comment: unimplemented
    AddToReset(bool), // [n], n%2==1 || n%2==0&&current_value%2==0: cell to 0
    ToLeft,           // [<]: Pointer to memory with 0
    ToRight,          // [>]: Pointer to memory with 0
    // [Move(n)], n != 1 | u16::MAX
    // [Add(n) Move(m) Add(l) Move(-(m+1))] | [Move(m) Add(l) Move(-(m+1)) Add(n)]: Cell to 0 and Cell in position m Set at n+l
    PointerStart(Option<usize>), // if a connection exists with the PointerEnd
    PointerEnd(Option<usize>),   // if a connection exists with the PointerStart
}

// Enum to represent the Brainfuck language commands
#[derive(Clone, PartialEq)]
pub enum Command {
    Add(u8),
    Move(u16),
    Buffer(BufferOptions),
    Loop(LoopOptions, usize), // loop function, index_file
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
                CommandClassic::OpenLoop(i) => Self::Loop(LoopOptions::PointerStart(None), *i),
                CommandClassic::ClosedLoop(i) => Self::Loop(LoopOptions::PointerEnd(None), *i),
            })
            .collect();

        tokens = Self::add_move_reduce_tokens(&tokens.clone());
        tokens = Self::loop_reduce_tokens(&tokens.clone());

        // Return the generated tokens
        Self::loop_conection(&tokens.clone())
    }

    fn add_move_reduce_tokens(commands: &Commands) -> Commands {
        let mut tokens: Commands = Vec::with_capacity(commands.capacity());
        let mut index = 0usize;

        loop {
            let token = commands.get(index);

            match token {
                Some(command) => match command {
                    Self::Add(_) => {
                        let (value, new_index) = Self::add_token(commands, index);
                        if value != 0 {
                            tokens.push(Self::Add(value));
                        }
                        index = new_index;
                        continue;
                    }
                    Self::Move(_) => {
                        let (value, new_index) = Self::move_token(commands, index);
                        tokens.push(Self::Move(value));
                        index = new_index;
                        continue;
                    }
                    command => tokens.push(command.clone()),
                },
                None => break,
            }

            index += 1;
        }

        tokens
    }

    fn loop_reduce_tokens(commands: &Commands) -> Commands {
        let mut tokens: Commands = Vec::with_capacity(commands.capacity());
        let mut index = 0usize;

        loop {
            let token = commands.get(index);

            match token {
                Some(command) => match command {
                    Self::Loop(LoopOptions::PointerStart(_), i) => {
                        let (value, new_index) = Self::loop_token(commands, index, *i);
                        tokens.push(value);
                        index = new_index;
                        continue;
                    }
                    Self::Loop(LoopOptions::PointerEnd(_), i) => {
                        tokens.push(Self::Loop(LoopOptions::PointerEnd(None), *i))
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

    fn loop_token(commands: &Commands, start: usize, index_file: usize) -> (Self, usize) {
        match commands.get(start + 1) {
            Some(Self::Loop(LoopOptions::PointerEnd(_), _)) => {
                (Self::Loop(LoopOptions::Comment, index_file), start + 2)
            }
            Some(Self::Add(value)) => match commands.get(start + 2) {
                Some(Self::Loop(LoopOptions::PointerEnd(_), _)) => (
                    Self::Loop(LoopOptions::AddToReset(*value % 2 == 0), index_file), // value is even
                    start + 3,
                ),
                _ => (
                    Self::Loop(LoopOptions::PointerStart(None), index_file),
                    start + 1,
                ),
            },
            Some(Self::Move(1)) => match commands.get(start + 2) {
                Some(Self::Loop(LoopOptions::PointerEnd(_), _)) => {
                    (Self::Loop(LoopOptions::ToRight, index_file), start + 3)
                }
                _ => (
                    Self::Loop(LoopOptions::PointerStart(None), index_file),
                    start + 1,
                ),
            },
            Some(Self::Move(u16::MAX)) => match commands.get(start + 2) {
                Some(Self::Loop(LoopOptions::PointerEnd(_), _)) => {
                    (Self::Loop(LoopOptions::ToLeft, index_file), start + 3)
                }
                _ => (
                    Self::Loop(LoopOptions::PointerStart(None), index_file),
                    start + 1,
                ),
            },
            _ => (
                Self::Loop(LoopOptions::PointerStart(None), index_file),
                start + 1,
            ),
        }
    }

    fn loop_conection(commands: &Commands) -> Result<Commands, InterpreterError> {
        let mut open_loop: Vec<(usize, usize)> = Vec::with_capacity(Self::token_counter(
            commands,
            Self::Loop(LoopOptions::PointerStart(None), 0),
        ));
        let mut loops: Vec<((usize, usize), (usize, usize))> =
            Vec::with_capacity(open_loop.capacity());

        for (index, token) in commands.iter().enumerate() {
            match token {
                Self::Loop(LoopOptions::PointerStart(_), i) => open_loop.push((index, *i)),
                Self::Loop(LoopOptions::PointerEnd(_), i) => match open_loop.pop() {
                    Some(open_index) => loops.push((open_index, (index, *i))),
                    None => return Err(InterpreterError::MalformedClosedLoop(*i)),
                },
                _ => continue,
            }
        }

        if !open_loop.is_empty() {
            return Err(InterpreterError::MalformedOpenLoop(open_loop[0].1));
        }

        let mut commands = commands.clone();
        while let Some((open_loop, closed_loop)) = loops.pop() {
            commands[open_loop.0] =
                Self::Loop(LoopOptions::PointerStart(Some(closed_loop.0)), open_loop.1);
            commands[closed_loop.0] =
                Self::Loop(LoopOptions::PointerEnd(Some(open_loop.0)), closed_loop.1);
        }

        Ok(commands)
    }

    pub fn token_counter(commands: &[Self], token: Self) -> usize {
        let mut counter = 0usize;

        for c in commands.iter() {
            match (c, token.clone()) {
                (
                    Self::Loop(LoopOptions::PointerStart(_), _),
                    Self::Loop(LoopOptions::PointerStart(_), _),
                ) => counter += 1,
                (
                    Self::Loop(LoopOptions::PointerEnd(_), _),
                    Self::Loop(LoopOptions::PointerEnd(_), _),
                ) => counter += 1,
                _ => {
                    if *c == token {
                        counter += 1;
                    }
                }
            }
        }

        counter
    }
}
