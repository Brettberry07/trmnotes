use std::io;

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
    text: String,
    exit: bool,
    explorer_open: bool,
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
            KeyCode::Backspace => {
                // remove the last character from the text
                self.text.pop();
            }
            KeyCode::Enter => {
                // append a newline character to the text
                self.text.push('\n');
            }
            _ => {
                // if the key is a character, append it to the text
                if let Some(c) = key_event.code.as_char() {
                    self.text.push(c);
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
            text: "".to_string(),
            exit: false,
            explorer_open: true,
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
            "<Ctrl+q> ".red().bold().into(),
            " Save ".bold().into(),
            "<Ctrl+s> ".green().bold().into(),
            " Toggle Explorer ".bold().into(),
            "<Ctrl+e> ".yellow().bold().into(),
        ]);

        // this is the text that will be displayed in the editor
        let editor_text = Text::from(self.text.clone());
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

        if self.explorer_open {
            files_block.render(chunks[0], buf);
            editor_block.render(chunks[1], buf);
        } else {
            // If explorer is closed, use the full area for the editor
            editor_block.render(area, buf);
        }
    }
}
