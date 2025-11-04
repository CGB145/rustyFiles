use chrono::DateTime;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::*;
use natord::compare;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListItem, ListState, Padding, Paragraph, Widget, Wrap
};
use ratatui::*;
use std::fmt::Alignment;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::str::FromStr;
use std::time::UNIX_EPOCH;
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
    error_output: Vec<String>,
    help:bool,
}

pub struct FileList {
    path: std::path::PathBuf,
    items: Vec<String>,
    selected_items: Vec<String>,
    state: ListState,
    is_file: bool,
    is_dir: bool,
    is_active: bool,
}

pub struct SelectedWidget {
    file_info: FileInfo,
    file_preview: FilePreview,
    file_list: FileList,
    file_selection: FileSelection,
}

pub struct FilePreview {
    is_active: bool,
    scroll: Scroll,
}

pub struct FileInfo {
    is_active: bool,
    scroll: Scroll,
}

pub struct FileSelection {
    is_active: bool,
    scroll: Scroll,
}

pub struct Scroll {
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
            error_output: vec![],
            help:false,
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
        }
    }
}

impl Default for SelectedWidget {
    fn default() -> Self {
        Self {
            file_info: FileInfo::default(),
            file_preview: FilePreview::default(),
            file_list: FileList::default(),
            file_selection: FileSelection::default(),
        }
    }
}

impl Default for FilePreview {
    fn default() -> Self {
        Self {
            is_active: false,
            scroll: Scroll::default(),
        }
    }
}

impl Default for FileInfo {
    fn default() -> Self {
        Self {
            is_active: false,
            scroll: Scroll::default(),
        }
    }
}

impl Default for FileSelection {
    fn default() -> Self {
        Self {
            is_active: false,
            scroll: Scroll::default(),
        }
    }
}

impl Default for Scroll {
    fn default() -> Self {
        Self { y: 0, x: 0 }
    }
}

impl FileList {
    fn update(self: &mut Self) {
        self.items.clear();

        let entries = match fs::read_dir(&self.path) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        let mut items: Vec<String> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path().to_string_lossy().to_string())
            .collect();

        //items.sort_by(|a, b| compare(a, b));
        self.items = items;
    }

    fn dir_next(self: &mut Self) {
        if let Some(index) = self.state.selected() {
            if let Some(entry) = self.items.get(index) {
                self.path = PathBuf::from(entry.as_str());
            }
        }

        if self.path.is_dir() {
            FileList::update(self);
            self.state.select(Some(0));
        }
    }

    fn dir_back(self: &mut Self) {
        self.path.pop();
        self.update();
        self.state.select(Some(0));
    }

    fn selected_item(self: &mut Self) -> String {
        let mut entry_name = String::new();

        if let Some(selected) = self.state.selected() {
            if let Some(item) = self.items.get(selected) {
                entry_name = item.split('/').last().unwrap_or("None").to_string();
                // use entry_name here
            }
        }
        entry_name
    }

    fn dir_ls(self: &mut Self)-> String{

        let path = self.items
                    .get(self.state.selected().unwrap())
                    .unwrap()
                    .as_str().to_string();                      

            let output = Command::new("bash")
                .arg("-c") // Use bash with -c to execute a command
                .arg(format!("ls {}", path))
                .output()
                .expect("failed to execute process");

                String::from_utf8_lossy(&output.stdout).to_string()
    }


}

impl SelectedWidget {
    fn change_widget(self: &mut Self) {
        if self.file_list.is_active {
            self.file_list.is_active = false;
            self.file_preview.is_active = true;
        } else if self.file_preview.is_active {
            self.file_preview.is_active = false;
            self.file_info.is_active = true;
        } else if self.file_info.is_active {
            self.file_info.is_active = false;
            self.file_selection.is_active = true;
        } else if self.file_selection.is_active {
            self.file_selection.is_active = false;
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
            KeyCode::Char('m') => self.move_files(),
            KeyCode::Char('c') => self.copy_files(),
            KeyCode::Char('d') => self.delete_files(),
            KeyCode::Char('h') => self.help = !self.help,
            KeyCode::Char('t') => self.error_output.push(self.notes.dir_ls()),
            KeyCode::Backspace => {
                self.notes.selected_items.clear();
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn previous(&mut self) {
        if self.selected_widget.file_list.is_active {
            let len = self.notes.items.len();
            if len == 0 {
                return;
            }

            let i = match self.notes.state.selected() {
                Some(0) | None => len - 1, // wrap to last
                Some(i) => i - 1,
            };

            self.selected_widget.file_preview.scroll = Scroll { y: (0), x: (0) };

            self.notes.state.select(Some(i));
        } else if self.selected_widget.file_preview.is_active
            && self.selected_widget.file_preview.scroll.y > 0
        {
            self.selected_widget.file_preview.scroll.y -= 1;
        } else if self.selected_widget.file_info.is_active
            && self.selected_widget.file_info.scroll.y > 0
        {
            self.selected_widget.file_info.scroll.y -= 1;
        } else if self.selected_widget.file_selection.is_active
            && self.selected_widget.file_selection.scroll.y > 0
        {
            self.selected_widget.file_selection.scroll.y -= 1;
        }
    }

    fn next(&mut self) {
        if self.selected_widget.file_list.is_active {
            let len = self.notes.items.len();
            if len == 0 {
                return;
            }

            let i = match self.notes.state.selected() {
                Some(i) if i >= len - 1 => 0, // wrap to start
                Some(i) => i + 1,
                None => 0, // nothing selected → start at 0
            };
            self.selected_widget.file_preview.scroll = Scroll { y: (0), x: (0) };
            self.notes.state.select(Some(i));
        } else if self.selected_widget.file_preview.is_active {
            self.selected_widget.file_preview.scroll.y += 1;
        } else if self.selected_widget.file_info.is_active {
            self.selected_widget.file_info.scroll.y += 1;
        } else if self.selected_widget.file_selection.is_active {
            self.selected_widget.file_selection.scroll.y += 1;
        }
    }

    fn open_via_app(&mut self) {
        let mut selection = String::new();
        selection.push_str(
            &self
                .notes
                .items
                .get(self.notes.state.selected().unwrap())
                .unwrap()
                .to_string(),
        );

        let output = Command::new("xdg-open")
            .arg(selection)
            .output()
            .expect("failed to execute process");

        let status = output.status;
        let stderr = String::from_utf8_lossy(&output.stderr);
        self.error_output.push(format!("{}, {}", status, stderr));
    }

    fn select_files(self: &mut Self) {
        let selection = &mut self.notes.selected_items;
        let selected_file = self
            .notes
            .items
            .get(self.notes.state.selected().unwrap())
            .unwrap()
            .as_str()
            .to_string();

        if selection.contains(&selected_file) {
            selection.retain(|item| item != &selected_file);
        } else {
            selection.push(selected_file);
        }
    }

    fn move_files(self: &mut Self) {
        let selection = &self.notes.selected_items;

        for item in selection {
            let output = Command::new("mv")
                .arg(item)
                .arg(self.notes.path.to_str().unwrap())
                .output()
                .expect("failed to execute process");

            let status = output.status;
            let stderr = String::from_utf8_lossy(&output.stderr);
            self.error_output.push(format!("{}, {}", status, stderr));
        }

        self.notes.update();
        self.notes.selected_items.clear();
    }

    fn copy_files(self: &mut Self) {
        let selection = &self.notes.selected_items;

        for item in selection {
            let output = Command::new("cp")
                .arg(item)
                .arg(self.notes.path.to_str().unwrap())
                .output()
                .expect("failed to execute process");

            let status = output.status;
            let stderr = String::from_utf8_lossy(&output.stderr);
            self.error_output.push(format!("{}, {}", status, stderr));
        }

        self.notes.update();
        self.notes.selected_items.clear();
    }

    fn delete_files(self: &mut Self) {
        let selection = &self.notes.selected_items;

        for item in selection {
            let output = Command::new("rm")
                .arg(item)
                .arg(self.notes.path.to_str().unwrap())
                .output()
                .expect("failed to execute process");

            let status = output.status;
            let stderr = String::from_utf8_lossy(&output.stderr);
            self.error_output.push(format!("{}, {}", status, stderr));
        }

        self.notes.update();
        self.notes.selected_items.clear();
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let selected_item = self.notes.state.selected();

        let mut path = "";

        if let Some(index) = self.notes.state.selected() {
            if let Some(item) = self.notes.items.get(index) {
                if let Some(pos) = item.rfind('/') {
                    path = &item[..pos]; // take everything up to the last '/'
                } else {
                    path = item; // no '/' found, keep as is
                }
            }
        }

        let border_color = if self.selected_widget.file_list.is_active {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_color)
            .title_bottom(Line::from(vec![
                Span::styled("h Help  ", Style::default().fg(Color::Cyan)).bold(),
                Span::styled("q Quit", Style::default().fg(Color::Red)).bold(),
            ]))
            .title(Line::from(Span::styled(
                "📁 File Browser",
                Style::default().fg(Color::Cyan).bold(),
            )))
            .title(path);

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
                let mut x = note.split('/').last().unwrap_or("Error").to_string();
                match fs::metadata(note){
                    Ok(metadata) => {
                        if metadata.is_dir(){
                            x.push('/');
                        }
                    }
                    _ =>{}
                }
                
                ListItem::new(x).style(style)
            })
            .collect();

        /*
            if i had no area...

        let size = crossterm::terminal::size();


        let rows: usize;
        if let Ok(size) = size {
            rows = (size.1 as f32 * 0.8) as usize;
        } else {
            rows = 0;
            self.error_output.push(format!("Couldnt get cols"));
        } */

        let rows = area.height as usize;
        let rows = rows.saturating_sub(2);

        if let Some(selected_index) = selected_item {
            let len = list_items.len();

            if selected_index >= len {
                self.notes.state.select(Some(0));
            }

            let x = ((len) as f32 / (rows) as f32).ceil() as i32;
            let multiples: Vec<i32> = (1..x).map(|i| rows as i32 * i).collect();

            for value in multiples {
                if selected_index >= value as usize {
                    list_items.drain(0..rows as usize);
                }
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

        let mut path = "";

        if let Some(index) = self.notes.state.selected() {
            if let Some(item) = self.notes.items.get(index) {
                path = item.as_str();
            }
        }

        /*
        this would panic
        let path = self.notes
        .items
        .get(self.notes.state.selected().unwrap())
        .unwrap()
        .as_str(); */

        if self.notes.is_file && !self.notes.is_dir {
            let file_content = match fs::read_to_string(&path) {
                Ok(content) => content,
                _ => String::from(&path.to_string()),
            };

            text.push_str(&file_content);
        } else {
                    let dir_items = self.notes.dir_ls();
                 let mut dir_items: Vec<String>  = dir_items
            .split('\n')
            .map(
                |item|{

                    let mut item = item.to_string();
                    let mut path = self.notes.selected_item();
                    path.push_str(format!("/{}",item).as_str());

                    let path_buf = PathBuf::from(path.clone());

                    if path_buf.is_dir(){
                        item.push('/');
                        item
                    }else{
                        item
                    }
   
                })
                .collect();
            dir_items.pop();
            text.push_str(format!("{}", dir_items.join("\n")).as_str());
        }

        let border_color = if self.selected_widget.file_preview.is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let scroll = (
            self.selected_widget.file_preview.scroll.y,
            self.selected_widget.file_preview.scroll.x,
        );



        let preview = Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .scroll(scroll)
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::styled("󰍉 Preview: ", Style::default().fg(Color::Cyan).bold()),
                        Span::raw(entry_name),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(border_color),
            );

        preview.render(area, buf);
    }

    fn render_file_info(&mut self, area: Rect, buf: &mut Buffer) {
        //let entry_name = self.notes.selected_item();

        let mut text = Text::raw(self.input.as_str());
        let mut file_data = String::new();
        if Some(self.notes.state.selected()).is_some() {
            if let Some(index) = self.notes.state.selected() {
                if let Some(path) = self.notes.items.get(index) {
                    let path_as_path = Path::new(path);
                    let extension: String;
                    match path_as_path.extension() {
                        Some(ext) => {extension=ext.to_string_lossy().to_string()}
                        None =>{extension=String::from("none")}
                    }
                    match fs::metadata(path) {
                        Ok(metadata) => {
                            self.notes.is_file = metadata.is_file();
                            self.notes.is_dir = metadata.is_dir();
                            file_data = format!(
                                "Extension:{:?}\nSize: {:.2} KiB\nCreated: {}\nModified: {}\n",
                                extension,
                                metadata.len() as f64 / 1024.0,
                                metadata
                                    .created()
                                    .ok()
                                    .and_then(|t| {
                                        let d = t.duration_since(UNIX_EPOCH).ok()?;
                                        DateTime::from_timestamp(
                                            d.as_secs() as i64,
                                            d.subsec_nanos(),
                                        )
                                    })
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| "Unavailable".to_string()),
                                metadata
                                    .modified()
                                    .ok()
                                    .and_then(|t| {
                                        let d = t.duration_since(UNIX_EPOCH).ok()?;
                                        DateTime::from_timestamp(
                                            d.as_secs() as i64,
                                            d.subsec_nanos(),
                                        )
                                    })
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| "Unavailable".to_string()),
                            );
                        }
                        _ => {}
                    }
                } else {
                    file_data = "No Permission for this Folder, PageDown to return".to_string();
                }
            } else {
                file_data = "No Permission for this Folder, PageDown to return".to_string();
            }

            text = Text::raw(file_data);
        }

        let border_color = if self.selected_widget.file_info.is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let scroll = (
            self.selected_widget.file_info.scroll.y,
            self.selected_widget.file_info.scroll.x,
        );

        let editor: Paragraph = Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .scroll(scroll)
            .block(
                Block::default()
                    .title(Line::from(Span::styled(
                        "ℹ File Info",
                        Style::default().fg(Color::Cyan).bold(),
                    )))
                    .borders(Borders::ALL)
                    .border_style(border_color),
            );

        editor.render(area, buf);
    }

    fn render_selection(&mut self, area: Rect, buf: &mut Buffer) {
        let text = self.notes.selected_items.join(", ");
        //let text = self.notes.selected_item();
        //let text = self.error_output.join(", ");



        /*let text = vec![
                    self.selected_widget.file_list.is_active.to_string(),
                    self.selected_widget.file_preview.is_active.to_string(),
                    self.selected_widget.file_info.is_active.to_string(),
                ];
                let test = String::from(text.join("\n"));


        */

        let border_color = if self.selected_widget.file_selection.is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let scroll = (
            self.selected_widget.file_selection.scroll.y,
            self.selected_widget.file_selection.scroll.x,
        );

        let selection = Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .title(Line::from(Span::styled(
                        "✅ Selected Files",
                        Style::default().fg(Color::Cyan).bold(),
                    )))
                    .title_bottom(Line::from(Span::styled(
                        "󰭜 Clear Selected Files",
                        Style::default().fg(Color::Cyan).bold(),
                    )))
                    .borders(Borders::ALL)
                    .style(border_color),
            )
            .scroll(scroll)
            .style(Style::default().fg(Color::Cyan));

        selection.render(area, buf);
    }

    fn render_help(&mut self, area: Rect, buf: &mut Buffer){

        let text = format!("↑↓ Navigate\n⏎ Open\n␣ Select\nPgUp/PgDn Dir Nav\nm Move\nc Copy\nd Delete\n󰭜 Clear Selected Files\nq Quit");
        let mut len_text: Vec<usize> = text.split('\n').map(|string| string.len()).collect();
        len_text.sort();
        let longest_text = len_text[len_text.len()-1];
        

        self.error_output.push(format!("{:?}",longest_text));
        let padding_top = (area.height - text.chars().filter(|&c| c == '\n').count() as u16)/2;
        let padding_left = area.width/2 - (longest_text/2) as u16;

        let test = Paragraph::new(text)
        .block(
            Block::default()
            .borders(Borders::ALL)
            .padding(Padding { left: (padding_left), right: (0), top: (padding_top), bottom: (0) })
        )
        .alignment(layout::HorizontalAlignment::Left);
        
        test.render(area, buf);
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

        let overlay = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(10), Constraint::Fill(1),Constraint::Percentage(10)])
            .split(area);



        if self.help{
            self.render_help(overlay[1], buf);
        }else{
            self.render_list(second_sub_layout[0], buf);
            self.render_file_info(sub_layout[1], buf);
            self.render_file_preview(sub_layout[0], buf);
            self.render_selection(second_sub_layout[1], buf);
        }
    }
}
