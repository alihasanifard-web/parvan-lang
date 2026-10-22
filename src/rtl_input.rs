use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};
use std::io::{self, stdout, Write};

pub struct RtlLineReader;

impl RtlLineReader {
    pub fn read_line(prompt: &str) -> io::Result<Option<String>> {
        let mut buffer: Vec<char> = Vec::new();
        let mut cursor_idx: usize = 0; // index in logical buffer

        terminal::enable_raw_mode()?;
        let mut out = stdout();

        Self::render(&mut out, prompt, &buffer, cursor_idx)?;

        loop {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                match code {
                    KeyCode::Enter => {
                        // Print newline in normal mode
                        terminal::disable_raw_mode()?;
                        println!();
                        let res: String = buffer.into_iter().collect();
                        return Ok(Some(res));
                    }
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        terminal::disable_raw_mode()?;
                        println!();
                        return Ok(None);
                    }
                    KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) && buffer.is_empty() => {
                        terminal::disable_raw_mode()?;
                        println!();
                        return Ok(None);
                    }
                    KeyCode::Char(ch) => {
                        buffer.insert(cursor_idx, ch);
                        cursor_idx += 1;
                        Self::render(&mut out, prompt, &buffer, cursor_idx)?;
                    }
                    KeyCode::Backspace => {
                        if cursor_idx > 0 {
                            cursor_idx -= 1;
                            buffer.remove(cursor_idx);
                            Self::render(&mut out, prompt, &buffer, cursor_idx)?;
                        }
                    }
                    KeyCode::Delete => {
                        if cursor_idx < buffer.len() {
                            buffer.remove(cursor_idx);
                            Self::render(&mut out, prompt, &buffer, cursor_idx)?;
                        }
                    }
                    KeyCode::Left => {
                        if cursor_idx > 0 {
                            cursor_idx -= 1;
                            Self::render(&mut out, prompt, &buffer, cursor_idx)?;
                        }
                    }
                    KeyCode::Right => {
                        if cursor_idx < buffer.len() {
                            cursor_idx += 1;
                            Self::render(&mut out, prompt, &buffer, cursor_idx)?;
                        }
                    }
                    KeyCode::Home => {
                        cursor_idx = 0;
                        Self::render(&mut out, prompt, &buffer, cursor_idx)?;
                    }
                    KeyCode::End => {
                        cursor_idx = buffer.len();
                        Self::render(&mut out, prompt, &buffer, cursor_idx)?;
                    }
                    _ => {}
                }
            }
        }
    }

    fn render(out: &mut io::Stdout, prompt: &str, buffer: &[char], _cursor_idx: usize) -> io::Result<()> {
        let text: String = buffer.iter().collect();
        let display_text = crate::persian::reshape_persian(&text);

        execute!(
            out,
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine)
        )?;

        print!("{}{}", prompt, display_text);
        out.flush()?;
        Ok(())
    }
}
