use chrono::DateTime;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::*;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::palette::material::{BLUE, GREEN};
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
    Widget, Wrap,
};
use ratatui::*;
use std::fmt::format;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};


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
    selected_widget: SelectedWidget,
}

pub struct FileList {
    path: std::path::PathBuf,
    items: Vec<String>,
    selected_items: Vec<String>,
    state: ListState,
    is_file: bool,
    is_dir: bool,
    is_active: bool,
    scroll: Scroll
}

pub struct SelectedWidget{
    file_info: FileInfo,
    file_preview: FilePreview,
    file_list: FileList,
    scroll: Scroll
}

pub struct FilePreview{
    is_active: bool,
    scroll: Scroll,
}

pub struct FileInfo{
    is_active: bool,
    scroll: Scroll,
}

pub struct Scroll{
    y: u16,
    x: u16,
}

impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            input: String::from(""),
            notes: FileList::default(),
            selected_widget: SelectedWidget::default(),
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
            selected_items: Vec::new(),
            is_file: false,
            is_dir: false,
            is_active: true,
            scroll: Scroll::default()
        }
    }
}

impl Default for SelectedWidget {
    fn default() -> Self {
        Self{
            file_info: FileInfo::default(),
            file_preview: FilePreview::default(),
            file_list: FileList::default(),
            scroll: Scroll::default(),
        }
    }
}

impl Default for FilePreview {
    fn default() -> Self {
        Self{
            is_active:false,
            scroll: Scroll::default(),
        }
    }
}

impl Default for FileInfo {
    fn default() -> Self {
        Self{
            is_active:false,
            scroll: Scroll::default(),
        }
    }
}


impl Default for Scroll{
    fn default() -> Self {
        Self{
            y: 0,
            x: 0,
        }
    }
}

impl FileList {
    fn update(self: &mut Self) {
        self.items.clear();

        let entries = match fs::read_dir(&self.path){
            Ok(entries) => entries,
            Err(_) => return,
        };

        for entry in entries.filter_map(Result::ok) {
            self.items.push(entry.path().to_string_lossy().to_string());
        }
    }

    fn dir_next(self: &mut Self) {

        if let Some(index) = self.state.selected() {
            if let Some(entry) = self.items.get(index) {
                self.path = PathBuf::from(entry.as_str());
            }
        }

        if self.path.is_dir() {
            FileList::update(self);
        }
    }

    fn dir_back(self: &mut Self) {
        self.path.pop();
        self.update();
    }

    fn selected_item(self: &mut Self) -> String {
        let mut entry_name = String::new();

        if let Some(selected) = self.state.selected() {
            if let Some(item) = self.items.get(selected) {
                entry_name = item
                    .split('/')
                    .last()
                    .unwrap_or("None")
                    .to_string();
                // use entry_name here
            }
        }
        entry_name
    }
}

impl SelectedWidget{
    fn change_widget(self: &mut Self) {
        if self.file_list.is_active{
            self.file_list.is_active = false;
            self.file_preview.is_active = true;
        }else if self.file_preview.is_active {
            self.file_preview.is_active = false;
            self.file_info.is_active = true;
        }else if self.file_info.is_active {
            self.file_info.is_active = false;
            self.file_list.is_active = true;
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

    fn draw(&mut self, frame: &mut Frame) {
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

    fn handle_key_events(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => self.previous(),
            KeyCode::Down => self.next(),
            // KeyCode::Char(c) => self.input.push(c),
            // KeyCode::PageUp => self.input.push_str(self.notes.items.join(" ").as_str()),
            KeyCode::End => self.input.push_str(self.notes.path.to_str().unwrap()),
            KeyCode::Insert => self.input.push_str(
                self.notes
                    .items
                    .get(self.notes.state.selected().unwrap())
                    .unwrap()
                    .as_str(),
            ),
            KeyCode::PageUp => self.notes.dir_next(),
            KeyCode::PageDown => self.notes.dir_back(),
            KeyCode::Enter => self.open_via_app(),
            KeyCode::Char(' ') => self.select_files(),
            KeyCode::Tab => self.selected_widget.change_widget(),
            KeyCode::Backspace => {
                self.input.pop();
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn previous(&mut self) {

        if self.selected_widget.file_list.is_active{
            if self.notes.state.selected().unwrap_or(0) == 0 {
                self.notes.state.select(Some(self.notes.items.len()));
            }
            self.notes.state.select_previous();
        }else if self.selected_widget.file_preview.is_active && self.selected_widget.file_preview.scroll.y > 0 {
            self.selected_widget.file_preview.scroll.y -= 1;
        }else if self.selected_widget.file_info.is_active && self.selected_widget.file_info.scroll.y > 0{
            self.selected_widget.file_info.scroll.y -= 1;
        }


    }

    fn next(&mut self) {
        if self.selected_widget.file_list.is_active{
            self.notes.state.select_next();
        }else if self.selected_widget.file_preview.is_active {
            self.selected_widget.file_preview.scroll.y += 1;
        }else if self.selected_widget.file_info.is_active{
            self.selected_widget.file_info.scroll.y += 1;
        }

    }

    fn open_via_app(&mut self) {
        let mut selection = String::from("xdg-open ");
        selection.push_str(
            &self
                .notes
                .items
                .get(self.notes.state.selected().unwrap())
                .unwrap()
                .to_string(),
        );

        let output = Command::new("bash")
            .arg("-c")
            .arg(selection)
            .status()
            .expect("failed to execute process");
    }

    fn select_files(self: &mut Self) {
        //ToDo
        let mut selection = &mut self.notes.selected_items;
        let selected_file = self.notes.items.get(self.notes.state.selected().unwrap()).unwrap().as_str().to_string();

        if selection.contains(&selected_file) {
            selection.retain(|item| item != &selected_file);
        }else{
            selection.push(selected_file);
        }

    }

    fn move_files(self: &mut Self) {
        //ToDo
    }

    fn copy_files(self: &mut Self) {
        //ToDo
    }

    fn delete_files(self: &mut Self) {
        //ToDo
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let selected_item = self.notes.state.selected();
        let item_info = selected_item.map(|i| i.to_string()).unwrap_or_default();

        let border_color = if self.selected_widget.file_list.is_active{
            Style::default().fg(Color::Blue)
        }else {
            Style::default()
        };

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_color)
            .title_bottom(Line::from(vec![
                Span::from("↑ select ↓").blue(),
                Span::raw("   "),
                Span::from("PageUp: Select Dir").blue(),
                Span::raw("   "),
                Span::from("PageDown: Back Dir").blue(),
                Span::raw("   "),
                Span::from("Enter: Open").blue(),
                Span::raw("   "),
                Span::from("Spacebar: Select").blue(),
            ]))
            .title(Line::from("q: Quit").red().bold());

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

        let list = List::new(list_items).block(block);
        list.render(area, buf);
    }

    fn render_file_preview(&mut self, area: Rect, buf: &mut Buffer) {


        let entry_name = self.notes.selected_item();

        let mut text = String::new();

        let path = self.notes
            .items
            .get(self.notes.state.selected().unwrap())
            .unwrap()
            .as_str();

        if self.notes.is_file && !self.notes.is_dir{
            let file_content = match fs::read_to_string(&path) {
                Ok(content) => content,
                _ => String::from(&path.to_string()),
            };

            text.push_str(&file_content);
        }else{
            text.push_str(&path);
        }

        let border_color = if self.selected_widget.file_preview.is_active{
            Style::default().fg(Color::Blue)
        }else {
            Style::default()
        };

        let scroll = (self.selected_widget.file_preview.scroll.y, self.selected_widget.file_preview.scroll.x);


        let preview = Paragraph::new(text).wrap(Wrap{trim: true}).scroll((scroll)).block(
            Block::default()
                .title(
                    Line::from("Preview").left_aligned()
                )
                .title(
                    Line::from(entry_name).centered()
                )
                .borders(Borders::ALL)
                .border_style(border_color)
        );




        preview.render(area, buf);
    }

    fn render_file_info(&mut self, area: Rect, buf: &mut Buffer) {

        let entry_name = self.notes.selected_item();


        let mut text = Text::raw(self.input.as_str());
        let mut file_data = String::new();
        if Some(self.notes.state.selected()).is_some() {
            if let Some(index) = self.notes.state.selected() {
                if let Some(path) = self.notes.items.get(index) {
                    match fs::metadata(path) {

                        Ok(metadata) => {
                            self.notes.is_file = metadata.is_file();
                            self.notes.is_dir = metadata.is_dir();
                           file_data = format!(
                                "Is File: {}\nIs Folder: {}\nCreated: {}\nModified: {} \n",
                                metadata.is_file(),
                                metadata.is_dir(),
                                metadata.created().ok().
                                    and_then(|t| {
                                        let d = t.duration_since(UNIX_EPOCH).ok()?;
                                        DateTime::from_timestamp(d.as_secs() as i64, d.subsec_nanos())

                                    })
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| "Unavailable".to_string()),
                                metadata.modified().ok().
                                    and_then(|t| {
                                        let d = t.duration_since(UNIX_EPOCH).ok()?;
                                        DateTime::from_timestamp(d.as_secs() as i64, d.subsec_nanos())

                                    })
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| "Unavailable".to_string()),
                           );


                        }
                        _ => {}
                    }
                }else {
                    file_data = "No Permission for this Folder, PageDown to return".to_string();
                }
            }else {
                file_data = "No Permission for this Folder, PageDown to return".to_string();
            }


           text = Text::raw(file_data);
        }

        let border_color = if self.selected_widget.file_info.is_active{
            Style::default().fg(Color::Blue)
        }else {
            Style::default()
        };

        let scroll = (self.selected_widget.file_info.scroll.y, self.selected_widget.file_info.scroll.x);

        let editor: Paragraph = Paragraph::new(text).wrap(Wrap{trim:true}).scroll((scroll)).block(
            Block::default()
                .title(Line::from(entry_name).centered())
                .borders(Borders::ALL)
                .border_style(border_color),
        );

        editor.render(area, buf);
    }

    fn render_debug(&mut self, area: Rect, buf: &mut Buffer) {
        //let text = self.notes.selected_items.join(", ");
        let text = vec![
            self.selected_widget.file_list.is_active.to_string(),
            self.selected_widget.file_preview.is_active.to_string(),
            self.selected_widget.file_info.is_active.to_string(),
        ];
        let test = String::from(text.join("\n"));

        let debug = Paragraph::new(test).wrap(Wrap{trim: true}).block(
            Block::default()
            .title(Line::from("Debug").centered())
            .borders(Borders::ALL),
        );

        debug.render(area, buf);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        let sub_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(layout[1]);

        let second_sub_layout = Layout::default()
        .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(layout[0]);

        self.render_list(second_sub_layout[0], buf);
        self.render_file_info(sub_layout[1], buf);
        self.render_file_preview(sub_layout[0], buf);
        self.render_debug(second_sub_layout[1], buf);

    }
}
