// COPYRIGHT (C) akettunen 2026
// program made by akettunen (me) & Claude
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    time::Duration,
};
use ratatui::crossterm::style::{Color as TermColor, Stylize};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

// orange color code and creation or sum
const ORANGE: Color = Color::Rgb(255, 140, 0);

// holds everything the player needs to remember between frames
struct App {
    tracks: Vec<PathBuf>,      // all audio files found in the playlist folderq
    current: usize,            // index of the track that is currently playing
    sink: Sink,                // rodio playback
    handle: OutputStreamHandle
}

impl App {
    // Creates the app and immediately starts playing the first track
    fn new(handle: OutputStreamHandle, tracks: Vec<PathBuf>) -> Self {
        let sink = Sink::try_new(&handle).expect("sound device couldnt be opened!");
        let mut app = App { tracks, current: 0, sink, handle };
        app.play_current();
        app
    }

    // open and play track
    fn play_current(&mut self) {
        if self.tracks.is_empty() {
            return; // nothing to play
        }
        let path = &self.tracks[self.current];
        if let Ok(file) = File::open(path) {
            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                if let Ok(sink) = Sink::try_new(&self.handle) {
                    sink.append(source);
                    self.sink = sink;
                }
            }
        }
    }

    // jump to next track
    fn next(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        self.current = (self.current + 1) % self.tracks.len();
        self.play_current();
    }

    // next track
    fn prev(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        self.current = (self.current + self.tracks.len() - 1) % self.tracks.len();
        self.play_current();
    }

    // pause toggle
    fn toggle_pause(&mut self) {
        if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }

    // name of playing music
    fn current_name(&self) -> String {
        self.tracks
            .get(self.current)
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

// subfolders not scanned
fn load_tracks(dir: &str) -> Vec<PathBuf> {
    let mut tracks: Vec<PathBuf> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    // supported file types
                    p.extension()
                        .map(|ext| {
                            matches!(
                                ext.to_string_lossy().to_lowercase().as_str(),
                                "mp3" | "wav" | "flac" | "ogg"
                            )
                        })
                        .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default(); // missing folder
    tracks.sort();
    tracks
}

// orange
fn orange_block(title: &str) -> Block<'_> {
    let style = Style::default().fg(ORANGE);
    Block::default()
        .title(title)
        .title_style(style)
        .borders(Borders::ALL)
        .border_style(style)
}

// drawing ui
fn draw(frame: &mut Frame, app: &App) {
    let base = Style::default().fg(ORANGE);

    // playlist now playing, and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // playlist
    let items: Vec<ListItem> = app
        .tracks
        .iter()
        .map(|p| ListItem::new(p.file_name().unwrap().to_string_lossy().to_string()))
        .collect();
    let list = List::new(items)
        .style(base)
        .block(orange_block("playlist"))
        // currently playing
        .highlight_style(base.add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    let mut state = ListState::default();
    if !app.tracks.is_empty() {
        state.select(Some(app.current)); // track name
    }
    frame.render_stateful_widget(list, chunks[0], &mut state);

    // playing
    let status = if app.sink.is_paused() { "> paused." } else { "> playing..." };
    let now_playing = Paragraph::new(format!("{}\n{}", status, app.current_name()))
        .style(base)
        .block(orange_block("statusq"));
    frame.render_widget(now_playing, chunks[1]);

    // help
    let help = Paragraph::new(r"space = pause/toggle | n = next | p = previous | q = quit | add music files to -> foxplayer-rs\src\music-here\")
        .style(base)
        .block(orange_block("controls & info "));
    frame.render_widget(help, chunks[2]);
}

fn main() -> std::io::Result<()> {
    let (_stream, handle) = OutputStream::try_default().expect("sound device not found!");

    // playlist folder
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/music-here/");
    let mut app = App::new(handle, load_tracks(dir));

    // terminal to tui mode
    let mut terminal = ratatui::init();

    loop {
        // redraw ui
        terminal.draw(|f| draw(f, &app))?;

        // input delay
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char(' ') => app.toggle_pause(),
                        KeyCode::Char('n') => app.next(),
                        KeyCode::Char('p') => app.prev(),
                        _ => {}
                    }
                }
            }
        }

        // automatic starting to next track
        if !app.tracks.is_empty() && app.sink.empty() {
            app.next();
        }
    }

    // terminal normal when exit
    // Restore the terminal to normal mode before exiting.
    ratatui::restore();

    //ascii logo at end (printed after restore so it stays visible)
    let fox = r"
    |\__/|      thank you for using foxplayer!
   /     \      + if you are interested in fox-like style, I have made a fox-style systemfetch https://github.com/akettunen/foxfetch-rs (Rust), https://github.com/akettunen/foxfetch (Cython)
  /_.~ ~,_\     - akettunen
     \@/";
    println!("{}", fox.with(TermColor::Rgb { r: 255, g: 140, b: 0 }));

    Ok(())
}