use std::io;
use std::io::{Read, Write};
use std::str;
use unicode_width::{UnicodeWidthStr, UnicodeWidthChar};
use termios::Termios;
use termios::tcsetattr;
use termios::{ECHO, ICANON, VTIME, VMIN, TCSANOW};
use libc::consts::os::posix88::STDIN_FILENO;
use super::CMD_PROMPT;
use super::{InputHandler, InputCmd};
use super::Key;

const UFT8_MASK: u8     = 0b_1100_0000;
const UFT8_LEAD: u8     = 0b_1100_0000;
const UTF8_CONTINUE: u8 = 0b_1000_0000;

const ESC_CHAR: u8 = 0x1B;
const UNKNOWN_ES: [u8; 2] = [ESC_CHAR, '[' as u8];
// Escape sequences for "normal" keys
const UP_ES:      [u8; 3] = [ESC_CHAR, '[' as u8, 'A' as u8];
const DOWN_ES:    [u8; 3] = [ESC_CHAR, '[' as u8, 'B' as u8];
const RIGHT_ES:   [u8; 3] = [ESC_CHAR, '[' as u8, 'C' as u8];
const LEFT_ES:    [u8; 3] = [ESC_CHAR, '[' as u8, 'D' as u8];
const HOME_ES:    [u8; 3] = [ESC_CHAR, 'O' as u8, 'H' as u8];
const END_ES:     [u8; 3] = [ESC_CHAR, 'O' as u8, 'F' as u8];
const PG_UP_ES:   [u8; 4] = [ESC_CHAR, '[' as u8, '5' as u8, '~' as u8];
const PG_DOWN_ES: [u8; 4] = [ESC_CHAR, '[' as u8, '6' as u8, '~' as u8];
const INSERT_ES:  [u8; 4] = [ESC_CHAR, '[' as u8, '2' as u8, '~' as u8];
const DELETE_ES:  [u8; 4] = [ESC_CHAR, '[' as u8, '3' as u8, '~' as u8];
// Escape sequences for function keys
const F1_ES:      [u8; 3] = [ESC_CHAR, 'O' as u8, 'P' as u8];
const F2_ES:      [u8; 3] = [ESC_CHAR, 'O' as u8, 'Q' as u8];
const F3_ES:      [u8; 3] = [ESC_CHAR, 'O' as u8, 'R' as u8];
const F4_ES:      [u8; 3] = [ESC_CHAR, 'O' as u8, 'S' as u8];
const F5_ES:      [u8; 5] = [ESC_CHAR, '[' as u8, '1' as u8, '5' as u8, '~' as u8];
const F6_ES:      [u8; 5] = [ESC_CHAR, '[' as u8, '1' as u8, '7' as u8, '~' as u8];
const F7_ES:      [u8; 5] = [ESC_CHAR, '[' as u8, '1' as u8, '8' as u8, '~' as u8];
const F8_ES:      [u8; 5] = [ESC_CHAR, '[' as u8, '1' as u8, '9' as u8, '~' as u8];
const F9_ES:      [u8; 5] = [ESC_CHAR, '[' as u8, '2' as u8, '0' as u8, '~' as u8];
const F10_ES:     [u8; 5] = [ESC_CHAR, '[' as u8, '2' as u8, '1' as u8, '~' as u8];
const F11_ES:     [u8; 5] = [ESC_CHAR, '[' as u8, '2' as u8, '3' as u8, '~' as u8];
const F12_ES:     [u8; 5] = [ESC_CHAR, '[' as u8, '2' as u8, '4' as u8, '~' as u8];

#[derive(Debug)]
pub struct PosixInputHandler {
    byte_buf: [u8; 32],     // Byte buffer, which is filled when reading
    byte_count: usize,      // Number of bytes used in the byte buffer
    line_hist: Vec<String>, // The line history
    line_buf: Vec<String>,  // An editable buffer of the previous- and the current line
    line_idx: usize,        // The index in the line buffer
    line_byte_pos: usize,   // The byte position in the current line
    cursor_pos: usize,      // The cursor position in the current line
    orig_termios: Option<Termios>,
}

impl PosixInputHandler {
    pub fn new() -> PosixInputHandler {
        let mut out = PosixInputHandler {
            byte_buf: [0; 32],
            byte_count: 0,
            line_hist: Vec::new(),
            line_buf: Vec::new(),
            line_idx: 0,
            line_byte_pos: 0,
            cursor_pos: 0,
            orig_termios: None,
        };
        out.line_buf.push(String::new());
        out
    }

    /// Blocks while waiting for the user to press a key
    fn poll_keypress(&mut self) -> Key {
        if self.byte_count == 0 {
            self.poll_stdin();
        }
        let byte = self.byte_buf[0];
        let (key, byte_len) = match byte {
            ESC_CHAR => self.parse_esc_seq(),
            0x7F => (Key::Backspace, 1), // Yes backspace is mapped to DEL
            0x09 => (Key::Tab, 1),
            0x0A => (Key::Enter, 1),
            0x20...0x7E => (Key::Char(byte as char), 1), // printable ASCII
            byte if is_utf8_lead(byte) => self.parse_utf8_char(), // utf8 codepoint
            // We don't know, so consume this byte and let the caller deal with it
            _ => (Key::Unknown, 1),
        };
        self.consume_buffer(byte_len);
        key
    }

    /// Blocks while populating `self.byte_buf` with a chunk of bytes from stdin
    fn poll_stdin(&mut self) {
        let read = io::stdin().read(&mut self.byte_buf[self.byte_count..])
            .ok()
            .expect("Could not read from terminal");
        self.byte_count += read;
    }

    fn parse_esc_seq(&self) -> (Key, usize) {
        // as of now these are the only sequences we deal with
        match self.byte_buf {
            // normal keys
            buf if buf.starts_with(&UP_ES) => (Key::Up, UP_ES.len()),
            buf if buf.starts_with(&DOWN_ES) => (Key::Down, DOWN_ES.len()),
            buf if buf.starts_with(&RIGHT_ES) => (Key::Right, RIGHT_ES.len()),
            buf if buf.starts_with(&LEFT_ES) => (Key::Left, LEFT_ES.len()),
            buf if buf.starts_with(&HOME_ES) => (Key::Home, HOME_ES.len()),
            buf if buf.starts_with(&END_ES) => (Key::End, END_ES.len()),
            buf if buf.starts_with(&PG_UP_ES) => (Key::PgUp, PG_UP_ES.len()),
            buf if buf.starts_with(&PG_DOWN_ES) => (Key::PgDown, PG_DOWN_ES.len()),
            buf if buf.starts_with(&INSERT_ES) => (Key::Insert, INSERT_ES.len()),
            buf if buf.starts_with(&DELETE_ES) => (Key::Delete, DELETE_ES.len()),
            // function keys
            buf if buf.starts_with(&F1_ES) => (Key::F(1), F1_ES.len()),
            buf if buf.starts_with(&F2_ES) => (Key::F(2), F2_ES.len()),
            buf if buf.starts_with(&F3_ES) => (Key::F(3), F3_ES.len()),
            buf if buf.starts_with(&F4_ES) => (Key::F(4), F4_ES.len()),
            buf if buf.starts_with(&F5_ES) => (Key::F(5), F5_ES.len()),
            buf if buf.starts_with(&F6_ES) => (Key::F(6), F6_ES.len()),
            buf if buf.starts_with(&F7_ES) => (Key::F(7), F7_ES.len()),
            buf if buf.starts_with(&F8_ES) => (Key::F(8), F8_ES.len()),
            buf if buf.starts_with(&F9_ES) => (Key::F(9), F9_ES.len()),
            buf if buf.starts_with(&F10_ES) => (Key::F(10), F10_ES.len()),
            buf if buf.starts_with(&F11_ES) => (Key::F(11), F11_ES.len()),
            buf if buf.starts_with(&F12_ES) => (Key::F(12), F12_ES.len()),
            // unknown escape sequence
            buf if buf.starts_with(&UNKNOWN_ES) => (Key::Unknown, UNKNOWN_ES.len()),
            // we didn't match any escape sequence, so we assume it is the escape key
            _ => (Key::Esc, 1),
        }
    }

    fn parse_utf8_char(&mut self) -> (Key, usize) {
        let mut bytes = Vec::new();
        let mut char_len = 2; // since we are bothering to parse ut8, the char is at least 2 bytes
        let lead_byte = self.byte_buf[0];

        // get the length of the utf8 char from the lead byte
        for i in char_len..8 {
            if lead_byte & (0b_1000_0000 >> i) == 0 {
                break
            } else {
                char_len += 1;
            }
        }
        let mut idx = 0;
        for _ in 0..char_len {
            if idx >= self.byte_count {
                // no more bytes, but we are not done, so poll some more
                self.byte_count = 0;
                self.poll_stdin();
                idx = 0;
            }
            bytes.push(self.byte_buf[idx]);
            idx += 1;
        }
        // now we try to parse it to a char
        match str::from_utf8(&bytes[..]) {
            Ok(s) => (Key::Char(s.chars().next().unwrap()), char_len),
            Err(_) => (Key::Unknown, char_len),
        }
    }

    /// Consumes `count` bytes from the front of the the buffer
    ///
    /// The first `count` bytes are removed from the buffer by moving the rest of the bytes
    /// forwards.
    fn consume_buffer(&mut self, count: usize) {
        for i in 0..self.byte_count {
            self.byte_buf[i] = if i + count < self.byte_buf.len() {
                self.byte_buf[i+count]
            } else {
                0
            };
        }
        self.byte_count -= count;
    }

    /// Moves `line_byte_pos` forward so it points to the next utf8 codepoint
    fn to_next_char(&mut self) ->  char {
        let start = self.line_byte_pos;
        self.line_byte_pos += 1;
        while self.line_byte_pos < self.line_byte_len() &&
              is_utf8_continue(self.line_byte_at(self.line_byte_pos)) {
            self.line_byte_pos += 1;
        }
        let bytes = self.line_buf[self.line_idx][start..self.line_byte_pos].as_bytes();
        unsafe {
            // Since the line buffer only contains valid utf8, there is no need to verify it again
            // before turning it into a strin
            str::from_utf8_unchecked(bytes).chars().next().unwrap()
        }
    }

    /// Moves `line_byte_pos` backwards so it points to the previous utf8 codepoint
    ///
    /// # Panics
    /// This function panics if the current line ends before the previous utf8 codepoint
    fn to_prev_char(&mut self) -> char {
        let end = self.line_byte_pos;
        self.line_byte_pos -= 1;
        while is_utf8_continue(self.line_byte_at(self.line_byte_pos)) {
            self.line_byte_pos -= 1;
        }
        let bytes = self.line_buf[self.line_idx][self.line_byte_pos..end].as_bytes();
        unsafe {
            // Since the line buffer only contains valid utf8, there is no need to verify it again
            // before turning it into a strin
            str::from_utf8_unchecked(bytes).chars().next().unwrap()
        }
    }

    /// Returns the `u8` at `idx`
    ///
    /// # Panics
    /// This function panics if either `line_buf` or the element looked at in `line_buf` is empty
    fn line_byte_at(&self, idx: usize) -> u8 {
        let bytes = self.line_buf[self.line_idx].as_bytes();
        bytes[idx]
    }

    /// Returns the length of the current line in bytes
    ///
    /// # Panics
    /// This function panics if `line_buf` is empty
    fn line_byte_len(&self) -> usize {
        self.line_buf[self.line_idx].len()
    }

    /// Returns the length of the current line in chars
    ///
    /// Note: this is O(n) as it is done by looping over all the chars in the line
    /// # Panics
    /// This function panics if `line_buf` is empty,
    /// or the line contains more than `usize::MAX` chars.
    fn line_column_len(&self) -> usize {
        self.line_buf[self.line_idx].width()
    }

}

fn is_utf8_lead(byte: u8) -> bool {
    byte & UFT8_MASK == UFT8_LEAD
}

fn is_utf8_continue(byte: u8) -> bool {
    byte & UFT8_MASK == UTF8_CONTINUE
}

impl InputHandler for PosixInputHandler {
    fn start(&mut self) -> io::Result<()> {
        // Only start if we are not already running
        if self.orig_termios.is_none() {
            let mut termios = try!(Termios::from_fd(STDIN_FILENO));
            // Save current state, for later restoration
            self.orig_termios = Some(termios.clone());
            // Enable raw mode so we can read keypress by keypress,
            // and turn off echoing, so characters aren't shown as they are typed.
            termios.c_lflag &= !(ECHO | ICANON);
            // Make reading block untill we get at least 1 byte
            termios.c_cc[VTIME] = 0;
            termios.c_cc[VMIN] = 1;
            // Here we go! Apply the new settings...
            try!(tcsetattr(STDIN_FILENO, TCSANOW, &termios));
        }
        Ok(())
    }

    fn stop(&mut self) -> io::Result<()> {
        // Only stop if we are currently running
        if let Some(orig_termios) = self.orig_termios {
            // Try to restore the original termios settings
            try!(tcsetattr(STDIN_FILENO, TCSANOW, &orig_termios));
        }
        Ok(())
    }

    fn handle_input(&mut self) -> InputCmd {
        match self.poll_keypress() {
            Key::Esc => InputCmd::Quit,
            Key::Enter => {
                let cmd = self.line_buf[self.line_idx].clone();
                if cmd == "quit" {
                    InputCmd::Quit
                } else {
                    self.line_hist.push(cmd.clone());
                    self.line_buf = self.line_hist.clone();
                    self.line_buf.push(String::new());
                    self.line_idx = self.line_buf.len() - 1;
                    self.line_byte_pos = 0;
                    self.cursor_pos = 0;
                    println!(""); // go to new line to prepare for output
                    InputCmd::Equation(cmd)
                }
            },
            Key::Backspace => {
                if self.line_byte_pos > 0 {
                    self.to_prev_char();
                    self.line_buf[self.line_idx].remove(self.line_byte_pos);
                    self.cursor_pos -= 1;
                }
                InputCmd::None
            },
            Key::Delete => {
                if self.line_byte_pos < self.line_byte_len() {
                    self.line_buf[self.line_idx].remove(self.line_byte_pos);
                }
                InputCmd::None
            },
            Key::Up => {
                if self.line_idx > 0 {
                    self.line_idx -= 1;
                    self.line_byte_pos = self.line_byte_len();
                    self.cursor_pos = self.line_column_len();
                }
                InputCmd::None
            },
            Key::Down => {
                if self.line_idx < self.line_buf.len() - 1{
                    self.line_idx += 1;
                    self.line_byte_pos = self.line_byte_len();
                    self.cursor_pos = self.line_column_len();
                }
                InputCmd::None
            },
            Key::Right => {
                if self.cursor_pos < self.line_column_len() {
                    let ch = self.to_next_char();
                    self.cursor_pos += ch.width().unwrap_or(0);
                }
                InputCmd::None
            },
            Key::Left => {
                if self.cursor_pos > 0 {
                    let ch = self.to_prev_char();
                    self.cursor_pos -= ch.width().unwrap_or(0);
                }
                InputCmd::None
            },
            Key::Home => {
                self.line_byte_pos = 0;
                self.cursor_pos = 0;
                InputCmd::None
            },
            Key::End => {
                self.line_byte_pos = self.line_byte_len();
                self.cursor_pos = self.line_column_len();
                InputCmd::None
            },
            Key::Char(ch) => {
                self.line_buf[self.line_idx].insert(self.line_byte_pos, ch);
                self.line_byte_pos += ch.len_utf8();
                self.cursor_pos += ch.width().unwrap_or(0);
                InputCmd::None
            },
            // For now we explicitly ignore these keys
            Key::Insert | Key::PgUp | Key::PgDown => InputCmd::None,
            _ => InputCmd::None,
        }
    }

    fn print_prompt(&self) {
        print!("\r\x1B[K"); // move back to the beginning of the line, and erase the old line
        print!("{}{}", CMD_PROMPT, self.line_buf[self.line_idx]); // print the current line
        print!("\r\x1B[{}C", self.cursor_pos + CMD_PROMPT.len()); // print the cursor
        // We explicitly call flush on stdout, or else the line won't be printed untill
        // after the user presses a key.
        io::stdout().flush().ok().expect("Could not write prompt to terminal");
    }
}

impl Drop for PosixInputHandler {
    fn drop(&mut self) {
        if let Some(orig_termios) = self.orig_termios {
            // This must succeed, or the terminal is screwed, which means there is no point in
            // continuing to run
            tcsetattr(STDIN_FILENO, TCSANOW, &orig_termios)
                .ok()
                .expect("Could not restore terminal settings");
        }
    }
}