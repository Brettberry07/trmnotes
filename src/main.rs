use std::{io, vec};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Layout, Constraint, Direction},
    style::Stylize,
    symbols::{block, border},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

pub struct App {
    text: Vec<String>,
    exit: bool,
    explorer_open: bool,
    cursor_x: usize,
    cursor_y: usize,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('s') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                println!("saved");
            }
            KeyCode::Char('e') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.explorer_open = !self.explorer_open;
            }
            KeyCode::Char('q') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.exit = true;
            }

            // handling cursor movement
            KeyCode::Left => {
                // move cursor left
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                } else if self.cursor_y > 0 {
                    // if cursor_x is 0 and cursor_y is greater than 0, go to previous line
                    self.cursor_y -= 1;
                    self.cursor_x = self.text[self.cursor_y].len(); // move cursor to the end of the previous line
                }
            }
            KeyCode::Right => {
                // move cursor right
                if self.cursor_x < self.text[self.cursor_y].len() {
                    self.cursor_x += 1;
                } else if self.cursor_y < self.text.len() - 1 {
                    // if cursor_x is at the end of the line and cursor_y is not the last line, go to next line
                    self.cursor_y += 1;
                    self.cursor_x = 0; // move cursor to the start of the next line
                }
            }
            KeyCode::Up => {
                // move cursor up
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                    if self.cursor_x > self.text[self.cursor_y].len() {
                        self.cursor_x = self.text[self.cursor_y].len(); // move cursor to the end of the previous line
                    }
                }
            }
            KeyCode::Down => {
                // move cursor down
                if self.cursor_y < self.text.len() - 1 {
                    self.cursor_y += 1;
                    if self.cursor_x > self.text[self.cursor_y].len() {
                        self.cursor_x = self.text[self.cursor_y].len(); // move cursor to the end of the next line
                    }
                }
            }

            // handling text editing    
            KeyCode::Backspace => {
                // remove the last character from the text
                if self.cursor_x > 0 && self.cursor_y < self.text.len() {
                    self.text[self.cursor_y].remove(self.cursor_x - 1);
                    self.cursor_x -= 1;
                } else if (self.cursor_x == 0) && (self.cursor_y > 0) {
                    // if cursor_x is 0 and cursor_y is greater than 0, go to precipous line
                    self.cursor_y -= 1;
                    self.cursor_x = self.text[self.cursor_y].len(); // move cursor to the end of the previous line
                }
            }
            KeyCode::Enter => {
                // split the current line at the cursor position
                let mut current_line = self.text[self.cursor_y].clone();
                let new_line = current_line.split_off(self.cursor_x);
                self.text[self.cursor_y] = current_line; // update the current line
                self.text.insert(self.cursor_y + 1, new_line); // insert the new line after the current line
                // move the cursor to the start of the new line
                self.cursor_y += 1;
                self.cursor_x = 0;
            }
            _ => {
                // if the key is a character, append it to the text
                if let Some(c) = key_event.code.as_char() {
                    self.text[self.cursor_y].insert(self.cursor_x, c);
                    self.cursor_x += 1;

                    // Ensure the cursor does not go out of bounds
                    if self.cursor_x > self.text[self.cursor_y].len() {
                        self.cursor_x = self.text[self.cursor_y].len();
                    }
                    // Ensure the cursor_y does not go out of bounds
                    if self.cursor_y >= self.text.len() {
                        self.text.push("".to_string());
                    }
                }
            }
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }
        Ok(())
    }

    fn default() -> Self {
        App {
            text: vec!["".to_string()],
            exit: false,
            explorer_open: true,
            cursor_x: 0,
            cursor_y: 0,
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // The block that holds everything
        let title = Line::from(" Trmnotes ".bold().blue());
        let main_block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        // Split the area into left and right panels
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(15), Constraint::Percentage(85)])
            .split(area);

        // Block on the left, this displays the files
        let files_block = Block::bordered()
            .title(" Files ".bold().blue())
            .border_set(border::PLAIN);

        // Block on the right, this displays the content of the file and the editor
        let instructions = Line::from(vec![
            " Quit ".bold().into(),
            "<Ctrl+Q> ".red().bold().into(),
            " Save ".bold().into(),
            "<Ctrl+S> ".green().bold().into(),
            " Toggle Explorer ".bold().into(),
            "<Ctrl+E> ".yellow().bold().into(),
            " Cursor Pos <".bold().into(),
            self.cursor_x.to_string().blue().bold().into(),
            " : ".bold().into(),
            self.cursor_y.to_string().blue().bold().into(),
            ">".bold().into(),
        ]);

        // this is the text that will be displayed in the editor
        let editor_text = Text::from(self.text.iter().map(|line| Line::from(line.as_str())).collect::<Vec<Line>>());
        let editor_paragraph = Paragraph::new(editor_text)
            .block(Block::default().borders(ratatui::widgets::Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });

        let editor_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1)])
            .split(if self.explorer_open { chunks[1] } else { area });
        // Render the editor paragraph in the bottom part of the right panel
        editor_paragraph.render(editor_area[0], buf);

        let editor_block = Block::bordered()
            .title(" Editor ".bold().blue())
            .title_bottom(instructions.centered())
            .border_set(border::PLAIN);

        // Render all together now
        main_block.render(area, buf);

        // rendering the cursor position
        let cursor_position = format!("Cursor: ({}, {})", self.cursor_x + 1, self.cursor_y + 1);
        buf.set_string(
            area.x + area.width - cursor_position.len() as u16 - 1,
            area.y + area.height - 1,
            cursor_position,
            ratatui::style::Style::default().fg(ratatui::style::Color::White),
        );

        if self.explorer_open {
            files_block.render(chunks[0], buf);
            editor_block.render(chunks[1], buf);

        } else {
            // If explorer is closed, use the full area for the editor
            editor_block.render(area, buf);
        }
    }
}
