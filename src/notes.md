**App Flow:**
1. Start app
2. Scan `./notes/` folder for `.md` files
3. Display file list (left pane)
4. Load selected file content into editable buffer (right pane)
5. On user input:
    - Update in-memory buffer
    - Mark file as "dirty"
6. After N seconds of inactivity:
    - Auto-save file (async)
7. Quit with `:q`, `Ctrl+C`, etc.

**Look:**
+--------------------+----------------------------------+
|   Notes List       |      Markdown Editor             |
|  - todo.md         |  # Shopping List                 |
|  - journal.md      |  - Apples                        |
|  > projects.md     |  - Bananas                       |
|                    |                                  |
|                    |  *(auto-saved)*                  |
+--------------------+----------------------------------+


**file structure:**
terminal-notes/
├── src/
│   ├── main.rs             # Main app loop
│   ├── ui.rs               # Layout & rendering logic (ratatui)
│   ├── input.rs            # Keyboard input handling
│   ├── notes.rs            # Note file loading/saving
│   └── config.rs           # App configuration (optional)
├── notes/                  # Your markdown notes go here
│   └── sample.md
├── Cargo.toml

**What I'm sarting with**

Start small:
1. Minimal Ratatui App

    Fullscreen terminal

    Two panes: note list (dummy data), editor (non-editable)

2. Add File Loading

    Scan notes/ dir

    Load selected file into string

3. Add Text Input

    Use tui-input or your own basic line buffer

    Allow basic editing: insert, backspace, move cursor

4. Add Async Saving

    Set a dirty flag on change

    After a delay (tokio::time::sleep), write the buffer back to file

5. Polish: Navigation + Visuals

    Use arrow keys or hjkl to move

    Add * next to unsaved files

    Add status bar (e.g., "Saved ✔" or "Unsaved ✱")


**.md stuff:**
| Task          | Markdown                              | Rendered Output              |
| ------------- | ------------------------------------- | ---------------------------- |
| Heading       | `# Heading 1`                         | **Heading 1**                |
| Bold          | `**bold text**`                       | **bold text**                |
| Italic        | `*italic text*`                       | *italic text*                |
| List          | `- Item 1` `- Item 2`                 | • Item 1<br>• Item 2         |
| Link          | `[OpenAI](https://openai.com)`        | [OpenAI](https://openai.com) |
| Image         | `![alt](img.png)`                     | *(renders image)*            |
| Code (inline) | `` `let x = 5;` ``                    | `let x = 5;`                 |
| Code block    | <pre>`rust<br>fn main() {}<br>`</pre> | *(renders as code)*          |


