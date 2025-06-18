use std::{path, vec};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Layout, Constraint, Direction},
    style::Stylize,
    symbols::{border},
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

/*
Explanation of the code:
This represents the app as a whole.
It contains all the logic for handling events, and the struct itself hold any variables we need across the whole app.
*/
pub struct App {
    text: Vec<String>,
    exit: bool,
    explorer_open: bool,
    cursor_x: usize,
    cursor_y: usize,
    folder: String,
    files: Vec<String>,
}

/*
Explanation of the code:
This is the implementation of the `App` struct.
Bascally this is where we can define all our methods and logic for the app.
This is where we handle the events, draw the UI, and run the app.
 */
impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            self.get_notes()?;
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
    /*
    Draws the Widget we rendered into the terminal. 
    Also draws the cursor at the current position.
     */
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());


        // render the cursor at the current position
        let cursor_position = Rect {
            x: if self.explorer_open {
                self.cursor_x as u16 + 35 // The 40 offset is required because of the left panel width and the border
            } else {
                self.cursor_x as u16 + 1 // if it's not open, we don't need the large offset
            },

            y: self.cursor_y as u16 + 1, // this is because of the border and title bar
            width: 1,
            height: 1,
        };
        frame.set_cursor_position((cursor_position.x, cursor_position.y));
    }

    /*
    This is where we can handle the key that is pressed.
    Each are handled through a match statement.
    We can handle combinations of keys.
    We have to handle certain keys seperately like arrow ketts, backspace, enter, etc.
      - Arrow keys allow us to move the cursor around the text.
      - Backspace allows us to delete the chararcter at the cursor pos
      - Enter allows us to split the current line at the cursor position.
    We also handle some special keys like Ctrl+S to save, Ctrl+E to toggle the explorer, and Ctrl+Q to quit.
    Every other key gets checked if it can be trasnlated to a char, if so we then just insert it to the text at the cursor position.
     */
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            // handling special key combinations
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
                }
            }
            KeyCode::Right => {
                // move cursor right
                if self.cursor_x < self.text[self.cursor_y].len() {
                    self.cursor_x += 1;
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
                } else if self.text[self.cursor_y].is_empty() && self.cursor_y > 0 {
                    // if the current line is empty and cursor_y is greater than 0, remove the current line and go to the previous line
                    self.text.remove(self.cursor_y);
                    self.cursor_y -= 1;
                    self.cursor_x = self.text[self.cursor_y].len(); // move cursor to the end of the previous line

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

    /*
    This is where we get all the events. We make sure that we only handle the key presses, 
    and then pass the key event to the `handle_key_event` method.
     */
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }
        Ok(())
    }

    // Getting all the files in folder and dealing with that stuff
    fn get_notes(&mut self) -> io::Result<()> {
        self.files.clear();
        for entry in fs::read_dir(&self.folder)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if let Some(file_name_str) = file_name.to_str() {
                        self.files.push(file_name_str.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    // Default state of the app
    fn default() -> Self {
        App {
            text: vec!["".to_string()],
            exit: false,
            explorer_open: true,
            cursor_x: 0,
            cursor_y: 0,
            folder: String::from("./notes"),
            files: vec![],
        }
    }
}

/*

Explanation of the code:
Bascially this is the rendering of the widget.
Nothing here is what "draws" it on the screen, but rather how it is structured.
This is the implementation of the `Widget` trait for the `App` struct.
Since we're implementing the `Widget` trait, we need to define the `render` method.
Now that we have the widget implemented we ccan turn our app struct into a widget.

*/
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        // Split the area into left and right panels
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(13), Constraint::Percentage(85), Constraint::Percentage(2),])
            .split(area);

        // Block on the left, this displays the files
        let files_paragraph = Paragraph::new(
            Text::from(self.files.iter().map(|file| Line::from(file.as_str())).collect::<Vec<Line>>())
        )
            .block(Block::default().borders(ratatui::widgets::Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });
        let files_block = Block::bordered()
            .title(" Files ".bold().blue())
            .border_set(border::PLAIN);
        let files_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1)])
            .split(chunks[0]);
        files_paragraph.render(files_area[0], buf);
        
        // Block on the right, this displays the content of the file and the editor
        let instructions = Line::from(vec![
            " Quit ".bold().into(),
            "<Ctrl+Q> ".red().bold().into(),
            " Save ".bold().into(),
            "<Ctrl+S> ".green().bold().into(),
            " Toggle Explorer ".bold().into(),
            "<Ctrl+E> ".yellow().bold().into(),
            " Cursor Pos <".bold().into(),

            if self.cursor_x == self.text[self.cursor_y].len() {
                self.cursor_x.to_string().red().bold().into()
            } else {
                self.cursor_x.to_string().blue().bold().into()
            },

            " : ".bold().into(),

            if self.cursor_y == self.text.len() - 1 {
                self.cursor_y.to_string().red().bold().into()
            } else {
                self.cursor_y.to_string().blue().bold().into()
            },

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

        // Rendering the line numbers on the left side
        // We create a vector of lines, each line is a number from 1 to the number of lines in the text
        let line_numbers: Vec<Line> = (0..self.text.len())
            .map(| i| {
                if i == self.cursor_y {
                    Line::from(i.to_string().red().bold())
                } else {
                    Line::from(i.to_string().blue().bold())
                }
            })
          //.map(|mut i| {
            //     if i == self.cursor_y {
            //         i = 0;
            //         Line::from(i.to_string().red().bold())    // This is for if I want the line number to be how far away fron the cursor it is
            //     } else {
            //         if i > self.cursor_y { i -= self.cursor_y; } else { i = self.cursor_y - i;}
            //         Line::from(i.to_string().blue().bold())
            //     }
            // })
            .collect();
        let line_numbers_text = Text::from(line_numbers);
        let line_numbers_paragraph = Paragraph::new(line_numbers_text)
            .block(Block::default().borders(ratatui::widgets::Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });
        line_numbers_paragraph.render(chunks[2], buf);

        if self.explorer_open {
            files_block.render(chunks[0], buf);
            editor_block.render(chunks[1], buf);

        } else {
            // If explorer is closed, use the full area for the editor
            editor_block.render(area, buf);
        }
    }
}
