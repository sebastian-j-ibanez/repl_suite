// Copyright (c) 2025 Sebastian Ibanez
// Author: Sebastian Ibanez
// Created: 2025-09-17

use std::fmt::Display;

use term_manager::TermManager;

/// Result type alias for repl_lib operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Function type for processing input lines.
pub type ProcessLineFunc = Box<dyn FnMut(String) -> Result<String>>;

/// Function type for determining if a line is complete.
pub type LineCompletionFunc = Box<dyn FnMut(String) -> bool>;

/// Error type for REPL operations.
#[derive(Debug)]
pub enum Error {
    InitFail(String),
    IoFlush(String),
    IoRead(String),
    IoWrite(String),
    ProcessLine(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InitFail(s) => write!(f, "initialization failed: {}", s),
            Error::IoFlush(s) => write!(f, "IO flush error: {}", s),
            Error::IoRead(s) => write!(f, "IO read error: {}", s),
            Error::IoWrite(s) => write!(f, "IO write error: {}", s),
            Error::ProcessLine(s) => write!(f, "Process Line error: {}", s),
        }
    }
}

/// Represents a single line of input with cursor position.
#[derive(Clone, Debug)]
pub struct Line {
    text: String,
    cursor_pos: usize,
}

impl Line {
    /// Creates a new empty line.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor_pos: 0,
        }
    }

    /// Inserts a character at the current cursor position.
    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }

    /// Removes the character before the cursor.
    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.text.remove(self.cursor_pos);
        }
    }

    /// Moves cursor one position to the left.
    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    /// Moves cursor one position to the right.
    pub fn move_right(&mut self) {
        if self.cursor_pos < self.text.len() {
            self.cursor_pos += 1;
        }
    }

    /// Returns the text content of the line.
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

/// Type of input being processed by the REPL.
#[derive(Copy, Clone, Debug)]
enum InputType {
    Normal,
    Escape,
    EscapeSequence,
}

/// Internal state for REPL operation flow.
#[derive(Copy, Clone, Debug)]
enum ReplState {
    Continue,
    Break,
}

/// Interactive Read-Eval-Print Loop implementation.
pub struct Repl {
    tmanager: TermManager,
    lines: Vec<Line>,
    current_line: usize,
    escape_buffer: Vec<u8>,
    input_state: InputType,
    process_line: ProcessLineFunc,
    is_line_complete: LineCompletionFunc,
    prompt: String,
    banner: String,
    welcome_msg: String,
}

impl Repl {
    /// Create a new REPL instance.
    ///
    /// ### Arguments
    ///
    /// * `prompt` - The prompt string to display
    /// * `banner` - Startup banner to display.
    /// * `welcome_msg` - Welcome message to display.
    /// * `process_line` - Function to process completed lines
    /// * `line_is_finished` - Function to determine if a line is terminated
    pub fn new(
        prompt: String,
        banner: String,
        welcome_msg: String,
        process_line: ProcessLineFunc,
        line_is_terminated: LineCompletionFunc,
    ) -> Result<Self> {
        let tmanager = TermManager::new().or_else(|e| {
            let msg = format!("failed to initialized Repl: {}", e);
            Err(Error::InitFail(msg))
        })?;
        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::new());
        let current_line = 0;
        let escape_buffer = Vec::new();
        let input_state = InputType::Normal;

        Ok(Repl {
            tmanager,
            lines,
            current_line,
            escape_buffer,
            input_state,
            process_line,
            is_line_complete: line_is_terminated,
            prompt,
            banner,
            welcome_msg,
        })
    }

    /// Prints the welcome banner and message.
    pub fn print_welcome(&mut self) {
        println!("{}\n{}", self.banner, self.welcome_msg);
    }

    /// Prints the REPL prompt.
    pub fn print_prompt(&mut self) {
        print!("{}", self.prompt);
    }

    /// Gets a line by index from the history.
    pub fn get_line(&self, index: usize) -> Option<&Line> {
        self.lines.get(index)
    }

    /// Read and process input until a complete line is entered.
    pub fn process_input(&mut self) -> Result<String> {
        self.tmanager
            .flush()
            .map_err(|_| Error::IoFlush("unable to flush stdout".into()))?;

        let mut output: Option<String> = None;

        loop {
            let mut buf = [0u8; 1];
            self.tmanager
                .read(&mut buf)
                .map_err(|e| Error::IoRead(format!("error reading from stdin: {}", e)))?;
            let c = buf[0];

            self.input_state = match self.input_state {
                InputType::Escape => {
                    self.escape_buffer.push(c);
                    if c == b'[' {
                        InputType::EscapeSequence
                    } else {
                        self.escape_buffer.clear();
                        InputType::Normal
                    }
                }
                InputType::EscapeSequence => {
                    self.escape_buffer.push(c);
                    if self.escape_buffer.len() == 2 && self.escape_buffer[0] == b'[' {
                        let final_byte = c;
                        self.handle_escape_sequence(final_byte)?;
                        self.escape_buffer.clear();
                        InputType::Normal
                    } else {
                        InputType::EscapeSequence
                    }
                }
                InputType::Normal => match self.handle_normal_input(c)? {
                    ReplState::Break => {
                        let finished_line = self
                            .get_line(self.current_line.saturating_sub(1))
                            .map(|l| l.text.clone())
                            .unwrap_or_default();
                        output = Some((self.process_line)(finished_line)?);

                        self.lines.push(Line::new());
                        self.current_line = self.lines.len() - 1;

                        break;
                    }
                    ReplState::Continue => self.input_state,
                },
            };
        }

        Ok(output.unwrap_or_default())
    }

    /// Handles ANSI escape sequences (arrow keys).
    fn handle_escape_sequence(&mut self, c: u8) -> Result<()> {
        match c {
            b'A' => {
                // Up arrow: recall previous line in history
                if self.current_line > 0 {
                    self.current_line -= 1;
                    self.redraw_current_line()?;
                }
            }
            b'B' => {
                // Down arrow: recall next line in history
                if self.current_line + 1 < self.lines.len() {
                    self.current_line += 1;
                    self.redraw_current_line()?;
                } else {
                    self.lines.push(Line::new());
                    self.current_line = self.lines.len() - 1;
                    self.redraw_current_line()?;
                }
            }
            b'C' => {
                // Right arrow
                if let Some(line) = self.lines.get_mut(self.current_line) {
                    line.move_right();
                    self.redraw_current_line()?;
                }
            }
            b'D' => {
                // Left arrow
                if let Some(line) = self.lines.get_mut(self.current_line) {
                    line.move_left();
                    self.redraw_current_line()?;
                }
            }
            _ => {}
        }

        self.escape_buffer.clear();
        self.input_state = InputType::Normal;

        Ok(())
    }

    /// Handles normal character input and control characters.
    fn handle_normal_input(&mut self, c: u8) -> Result<ReplState> {
        let current_line = self
            .lines
            .get_mut(self.current_line)
            .ok_or_else(|| Error::ProcessLine("no active line".into()))?;

        match c {
            b'\n' | b'\r' => {
                // Newline/enter line
                if (self.is_line_complete)(current_line.text.clone()) {
                    println!();
                    Ok(ReplState::Break)
                } else {
                    current_line.insert_char('\n');
                    Ok(ReplState::Continue)
                }
            }
            0x7F => {
                // Backspace
                current_line.backspace();
                self.redraw_current_line()?;
                Ok(ReplState::Continue)
            }
            0x01 => {
                // Ctrl-A = move to line start
                current_line.cursor_pos = 0;
                self.redraw_current_line()?;
                Ok(ReplState::Continue)
            }
            0x05 => {
                // Ctrl-E = move to line end
                current_line.cursor_pos = current_line.text.len();
                self.redraw_current_line()?;
                Ok(ReplState::Continue)
            }
            0x1B => {
                // Escape
                self.input_state = InputType::Escape;
                Ok(ReplState::Continue)
            }
            c if c.is_ascii_control() => Ok(ReplState::Continue),
            c => {
                current_line.insert_char(c as char);
                self.redraw_current_line()?;
                Ok(ReplState::Continue)
            }
        }
    }

    /// Redraws the current line with proper cursor positioning.
    fn redraw_current_line(&mut self) -> Result<()> {
        let line = self
            .lines
            .get(self.current_line)
            .ok_or_else(|| Error::ProcessLine("no active line for redraw".into()))?;

        print!("\r{}{}\x1b[K", self.prompt, line.text);
        let right_after_prompt = self.prompt.len() + line.cursor_pos;
        let total_len = self.prompt.len() + line.text.len();
        if total_len > right_after_prompt {
            print!("\x1b[{}D", total_len - right_after_prompt);
        }

        self.tmanager
            .flush()
            .map_err(|_| Error::IoFlush("unable to flush stdout".into()))?;
        Ok(())
    }
}
