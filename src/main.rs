use std::{default, path, vec};
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
    // vars related to text editing
    text: Vec<String>,                    // text that is displayed, one line is one string
    folder: String,                       // folder where notes are stored
    files: Vec<String>,                   // all the files in that folder
    current_file: Option<String>,         //current file that is being edited, if None, we use the default.txt

    // vars related to app state and menus
    exit: bool,                           // if true, stop running the app
    explorer_open: bool,                  // wehther or not we show the menu that displays the files
    help_menu_open: bool,                 // wehther or not we display some keybinds

    note_create_mode: bool,               // if true, we are in the mode to create a new note
    new_file_name: String,                // name of the new file that is being created, if empty, we use the default.txt

    file_select_mode: bool,
    file_select_index: usize,             // index of the file that is selected in the file explorer


    // vars related to cursor position
    cursor_x: usize,
    cursor_y: usize,

}

impl default::Default for App {
    // Default state of the app
    fn default() -> Self {
        App {
            text: vec!["".to_string()],
            folder: String::from("./notes/"),
            files: vec![],
            current_file: "default.txt".to_string().into(),

            exit: false,
            explorer_open: true,
            help_menu_open: false,

            note_create_mode: false,
            new_file_name: String::new(),

            file_select_mode: false,
            file_select_index: 0,

            cursor_x: 0,
            cursor_y: 0,

        }
    }
}

/*
Explanation of the code:
This is the implementation of the `App` struct.
Bascally this is where we can define all our methods and logic for the app.
This is where we handle the events, draw the UI, and run the app.
 */
impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.open_note( &self.current_file.clone().unwrap_or_else(|| "default.txt".to_string()))?;

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
        if self.note_create_mode {
            // If we are in note creation mode, we handle the key events differently
            if key_event.code == KeyCode::Enter {
                // If Enter is pressed, we create a new note with the current file name
                if !self.new_file_name.is_empty() {
                    let file_name = self.new_file_name.clone();
                    if let Err(e) = self.create_note(&file_name) {
                        eprintln!("Failed to create note: {}", e);
                    } else {
                        self.current_file = Some(file_name);
                        self.note_create_mode = false; // Exit note creation mode
                        self.new_file_name.clear();    // Clear the new file name
                    }
                }
            } else if key_event.code == KeyCode::Backspace {
                // If Backspace is pressed, remove the last character from the new file name
                if !self.new_file_name.is_empty() {
                    self.new_file_name.pop();
                }
            } else if key_event.code == KeyCode::Esc {
                // If Escape is pressed, exit note creation mode
                self.note_create_mode = false;
                self.new_file_name.clear(); // Clear the new file name
            } else if let Some(c) = key_event.code.as_char() {
                // If any other character is pressed, append it to the new file name
                self.new_file_name.push(c);
            }
            return; // Exit early if in note creation mode
        } else if self.file_select_mode {
            // If we are in file selection mode, we handle the key events differently
            if key_event.code == KeyCode::Enter {
                // If Enter is pressed, open the selected file
                if self.file_select_index < self.files.len() {
                    let file_name = &self.files[self.file_select_index].clone();
                    if let Err(e) = self.open_note(file_name) {
                        eprintln!("Failed to open note: {}", e);
                    } else {
                        self.current_file = Some(file_name.clone());
                        self.file_select_mode = false; // Exit file selection mode
                        self.file_select_index = 0; // Reset the file selection index
                    }
                }
            } else if key_event.code == KeyCode::Esc {
                // If Escape is pressed, exit file selection mode
                self.file_select_mode = false;
            } else if key_event.code == KeyCode::Up || key_event.code == KeyCode::Char('w') {
                // Move up in the file list
                if self.file_select_index > 0 {
                    self.file_select_index -= 1;
                }
            } else if key_event.code == KeyCode::Down || key_event.code == KeyCode::Char('s') {
                // Move down in the file list
                if self.file_select_index < self.files.len() - 1 {
                    self.file_select_index += 1;
                }
            }
            return; // Exit early if in file selection mode
        }

        match key_event.code {
            // handling special key combinations
            KeyCode::Char('s') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                if let Some(file_name) = self.current_file.clone() {
                    if let Err(e) = self.save_note(&file_name) {
                        eprintln!("Failed to save note: {}", e);
                    }
                }
            }
            KeyCode::Char('e') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.explorer_open = !self.explorer_open;
            }
            KeyCode::Char('q') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.exit = true;
            }
            KeyCode::Char('n') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                // create a new note
                // Inside this loop we are going to display a prompt for the user to enter the name of the new note.
                self.note_create_mode = true;
            }
            KeyCode::Char('o') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.file_select_mode = true;
                self.get_notes().expect("Failed to get notes");
            }
            KeyCode::Char('h') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                // toggle help menu
                self.help_menu_open = !self.help_menu_open;
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
        self.files.sort(); // Sort files alphabetically
        Ok(())
    }

    fn create_note(&mut self, file_name: &str) -> io::Result<()> {
        let file_path = Path::new(&self.folder).join(file_name);
        if !file_path.exists() {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(file_path)?;
            file.write_all(b"")?; // Create an empty file
            self.get_notes()?; // Refresh the list of files
        }
        Ok(())
    }

    fn open_note(&mut self, file_name: &str) -> io::Result<()> {
        let file_path = Path::new(&self.folder).join(file_name);
        if file_path.exists() {
            let mut file = File::open(file_path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            self.text = content.lines().map(|line| line.to_string()).collect();
            if self.text.is_empty() {
                self.text.push("".to_string()); // Ensure there's at least one line
            }
            self.cursor_x = 0;
            self.cursor_y = 0;
        } else {
            eprintln!("File not found: {}", file_name);
        }
        Ok(())
    }

    fn save_note(&mut self, file_name: &str) -> io::Result<()> {
        let file_path = Path::new(&self.folder).join(file_name);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)?;
        for line in &self.text {
            writeln!(file, "{}", line)?;
        }
        Ok(())
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
        
        // Block on the right, this displays the content of the file and the editor
        let instructions = Line::from(vec![
            " Help ".bold().into(),
            "<Ctrl+H> ".yellow().bold().into(),
            " Quit ".bold().into(),
            "<Ctrl+Q> ".red().bold().into(),
            // " Save ".bold().into(),
            // "<Ctrl+S> ".green().bold().into(),
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

            files_block.render(chunks[0], buf);
            editor_block.render(chunks[1], buf);

        } else {
            // If explorer is closed, use the full area for the editor
            editor_block.render(area, buf);
        }

        // Rendering the help menu if it's open
        if self.help_menu_open {
            // preparing help area
            // 1) determine size of the help box
            let help_width = 30;
            let help_height = 10;
            let x = (area.width.saturating_sub(help_width)) / 2 + area.x;
            let y = (area.height.saturating_sub(help_height)) / 2 + area.y;
            let help_area = Rect::new(x, y, help_width, help_height);

            // Manually clear the help area by filling it with spaces
            for y in help_area.top()..help_area.bottom() {
                for x in help_area.left()..help_area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol(" ");
                    }
                }
            }

            let help_text = Text::from(vec![
                Line::from("Ctrl+Q: Quit"),
                Line::from("Ctrl+S: Save"),
                Line::from("Ctrl+E: Toggle Explorer"),
                Line::from("Ctrl+N: Create New Note"),
                Line::from("Ctrl+O: Open Note"),
                Line::from("Ctrl+H: Toggle Help Menu"),
            ]);
            let help_paragraph = Paragraph::new(help_text)
                .block(Block::default().borders(ratatui::widgets::Borders::ALL).title(" Help ".bold().blue()))
                .wrap(ratatui::widgets::Wrap { trim: true });
            help_paragraph.render(help_area, buf);
        }

        // rednering the create note block if in note creation mode
        if self.note_create_mode {
            // preparing create note area
            let create_note_width = 35;
            let create_note_height = 8;
            let x = (area.width.saturating_sub(create_note_width)) / 2 + area.x;
            let y = (area.height.saturating_sub(create_note_height)) / 2 + area.y;
            let create_note_area = Rect::new(x, y, create_note_width, create_note_height);

            // Manually clear the create note area by filling it with spaces
            for y in create_note_area.top()..create_note_area.bottom() {
                for x in create_note_area.left()..create_note_area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol(" ");
                    }
                }
            }

            let create_note_text = Text::from(vec![
                Line::from("Create Note:"),
                Line::from(format!("Name: {}", self.new_file_name)),
                Line::from(""),
                Line::from(vec![
                    "Create: ".into(),
                    "Enter".bold().green().into(),
                    " | Cancel: ".into(),
                    "Esc".bold().red().into(),
                ]),
            ]);
            let create_note_paragraph = Paragraph::new(create_note_text)
                .block(Block::default().borders(ratatui::widgets::Borders::ALL).title(" Create Note ".bold().blue()))
                .wrap(ratatui::widgets::Wrap { trim: true });
            create_note_paragraph.render(create_note_area, buf);
        }

        // rendering the file selection mode if it's open
        if self.file_select_mode {
            // preparing file selection area
            let file_select_width = 40;
            let file_select_height = 4 + self.files.len() as u16; // 4 for the instructions + number of files
            let x = (area.width.saturating_sub(file_select_width)) / 2 + area.x;
            let y = (area.height.saturating_sub(file_select_height)) / 2 + area.y;
            let file_select_area = Rect::new(x, y, file_select_width, file_select_height);

            // Manually clear the file selection area by filling it with spaces
            for y in file_select_area.top()..file_select_area.bottom() {
                for x in file_select_area.left()..file_select_area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol(" ");
                    }
                }
            }

            // Prepare the text for the file selection menu
            let mut file_lines: Vec<Line> = self.files.iter().enumerate().map(|(i, file)| {
                if i == self.file_select_index {
                    Line::from(file.as_str().bold().yellow()) // Highlight the selected file
                } else if file.as_str() == self.current_file.as_deref().unwrap_or("default.txt") {
                    Line::from(file.as_str().bold().green()) // Highlight the current file
                } else {
                    Line::from(file.as_str())
                }
            }).collect();

            // Add instructions at the bottom
            file_lines.push(Line::from(""));
            file_lines.push(Line::from(vec![
                "Select: ".into(),
                "Enter".bold().green().into(),
                " | Cancel: ".into(),
                "Esc".bold().red().into(),
            ]));

            let file_select_text = Text::from(file_lines);
            let file_select_paragraph = Paragraph::new(file_select_text)
                .block(Block::default().borders(ratatui::widgets::Borders::ALL).title(" Select File ".bold().blue()))
                .wrap(ratatui::widgets::Wrap { trim: true });
            file_select_paragraph.render(file_select_area, buf);
        }
    }
}
