use ratatui::*;
use crossterm::*;
use std::{fs, io};
use std::process::Command;
use std::fs::File;
use std::process::exit;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::style::palette::material::{BLUE, GREEN};
use ratatui::style::palette::tailwind::SLATE;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, BorderType, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph, Widget, Wrap};



fn main() -> io::Result<()> {
    let mut terminal = init();
    let mut app = App::default();
    let app_result = app.run(&mut terminal);
    restore();

    app_result
}
pub struct App {
    exit: bool,
    input: String,
    notes: FileList,

}

pub struct FileList{
    path: std::path::PathBuf,
    items: Vec<String>,
    state: ListState,
}


impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            input: String::from(""),
            notes: FileList::default(),

        }
    }
}

impl Default for FileList {
    fn default() -> Self {
        let mut notes = Vec::new();

        let path = std::env::current_dir().unwrap(); // current directory

        for entry in fs::read_dir(&path).unwrap() {
            let entry = entry.unwrap();
                notes.push(entry.path().to_string_lossy().to_string());
        }

        Self {
            path,
            items: notes,
            state: ListState::default(),
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().expect("PANICCCCC");
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame){
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_events(key_event)
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_events(&mut self, key_event: KeyEvent){
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => self.previous(),
            KeyCode::Down => self.next(),
            // KeyCode::Char(c) => self.input.push(c),
            // KeyCode::PageUp => self.input.push_str(self.notes.items.join(" ").as_str()),
            KeyCode::PageUp => self.input.push_str(self.notes.path.to_str().unwrap()),
            KeyCode::PageDown => self.input.push_str(self.notes.items.get(self.notes.state.selected().unwrap()).unwrap().as_str()),
            KeyCode::End => {
                Command::new("bash")
                    .arg("-c")
                    .arg("cd ../")
                    .output()
                    .expect("Failed to execute command");
            }
            KeyCode::Enter => self.input.push('\n'),
            KeyCode::Backspace => {
                self.input.pop();
            },
            _ => {}
        }
    }

    fn exit(&mut self){
        self.exit = true;
    }

    fn next(&mut self){
        self.notes.state.select_next();
    }
    fn previous(&mut self){
        if self.notes.state.selected().unwrap_or(0) == 0 {
            self.notes.state.select(Some(self.notes.items.len()));
        }
        self.notes.state.select_previous();
    }


    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {

        let selected_item = self.notes.state.selected();
        let  item_info = selected_item.map(|i| i.to_string()).unwrap_or_default();


        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Line::from("↑ select ↓").left_aligned())
            .title(Line::from(item_info).centered());

        let mut list_items: Vec<ListItem> = self
            .notes
            .items
            .iter()
            .enumerate()
            .map(|(i, note)| {

                let style = if Some(i) == selected_item {
                    Style::default().fg(Color::Blue).bg(Color::White)
                } else {
                    Style::default()
                };
                ListItem::new(note.clone()).style(style)
            })
            .collect();

        let mut selected_index = selected_item.unwrap_or(0);
        let len = list_items.len();

        if let Some(mut selected_index) = selected_item {
            let len = list_items.len();

            if selected_index >= len {
                selected_index = 0;
                self.notes.state.select(Some(0));
            }

            if selected_index > 0 {
                list_items.drain(0..selected_index);
            }
        } else {
            self.notes.state.select(Some(0));
        }

        let list = List::new(list_items)
            .block(block);
        list.render(area, buf);
    }

    fn render_editor(&mut self, area: Rect, buf: &mut Buffer) {
        let text = Text::raw(self.input.as_str());
        let editor: Paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(Line::from("q to quit").left_aligned())
                    .title(Line::from("Middle Title").centered())
                    .title(Line::from("Right Title").right_aligned())
                    .borders(Borders::ALL)
            );

        editor.render(area, buf);
    }

}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer){

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(80)
            ])
            .split(area);


        self.render_list(layout[0], buf);
        self.render_editor(layout[1], buf);

    }
}

