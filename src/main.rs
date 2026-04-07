use std::{
    fs,
    io::{self, stdout},
    path::PathBuf,
    time::{Duration, Instant},
};

use chrono::{Local, NaiveDate};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::Rng;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph},
};
use serde::{Deserialize, Serialize};

// Tokyonight Night colors
mod colors {
    use ratatui::style::Color;
    pub const BG: Color = Color::Rgb(26, 27, 38);
    pub const FG: Color = Color::Rgb(169, 177, 214);
    pub const RED: Color = Color::Rgb(247, 118, 142);
    pub const GREEN: Color = Color::Rgb(158, 206, 106);
    pub const YELLOW: Color = Color::Rgb(224, 175, 104);
    pub const BLUE: Color = Color::Rgb(122, 162, 247);
    pub const MAGENTA: Color = Color::Rgb(187, 154, 247);
    pub const CYAN: Color = Color::Rgb(125, 207, 255);
    pub const COMMENT: Color = Color::Rgb(86, 95, 137);
}

// Pet types
#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
enum PetType {
    Blob,
    Cat,
    Robot,
    Ghost,
}

impl PetType {
    fn random() -> Self {
        match rand::thread_rng().gen_range(0..4) {
            0 => PetType::Blob,
            1 => PetType::Cat,
            2 => PetType::Robot,
            _ => PetType::Ghost,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            PetType::Blob => "Blob",
            PetType::Cat => "Cat",
            PetType::Robot => "Robot",
            PetType::Ghost => "Ghost",
        }
    }
}

// Pet evolution stages with ASCII art
mod pet {
    use super::PetType;

    // === EGGS (all types share similar eggs with slight variation) ===
    pub const EGG_BLOB: &[&str] = &["  ___  ", " /   \\ ", "|  ·  |", " \\___/ "];
    pub const EGG_BLOB_WOBBLE: &[&str] = &["  ___  ", " /   \\ ", "|  *  |", " \\___/ "];
    pub const EGG_CAT: &[&str] = &["  ___  ", " /^ ^\\ ", "|     |", " \\___/ "];
    pub const EGG_CAT_WOBBLE: &[&str] = &["  ___  ", " /^ ^\\ ", "|  ~  |", " \\___/ "];
    pub const EGG_ROBOT: &[&str] = &["  ___  ", " [   ] ", "|  #  |", " [___] "];
    pub const EGG_ROBOT_WOBBLE: &[&str] = &["  ___  ", " [   ] ", "|  @  |", " [___] "];
    pub const EGG_GHOST: &[&str] = &["  ___  ", " (   ) ", "|  ~  |", " \\^_^/ "];
    pub const EGG_GHOST_WOBBLE: &[&str] = &["  ___  ", " (   ) ", "|  o  |", " \\^_^/ "];

    // === BLOB ===
    pub const BLOB_BABY: &[&str] = &[" (^_^) ", "  /|\\  ", "   |   "];
    pub const BLOB_BABY_HAPPY: &[&str] = &[" (^o^) ", " \\|^|/ ", "   |   "];
    pub const BLOB_BABY_WORK: &[&str] = &[" (>_<) ", "  /|\\  ", "  /|   "];
    pub const BLOB_BABY_SLEEP: &[&str] = &[" (-_-) ", "  /|\\  ", "  z z  "];

    pub const BLOB_TEEN: &[&str] = &["  \\o/  ", "   |   ", "  / \\  "];
    pub const BLOB_TEEN_HAPPY: &[&str] = &[" \\(^o^)/", "   |   ", "  / \\  "];
    pub const BLOB_TEEN_WORK: &[&str] = &["  (•_•)", "  <|>  ", "  / \\  "];
    pub const BLOB_TEEN_SLEEP: &[&str] = &["  (_ _)", "  /|\\  ", " z/ \\z "];

    pub const BLOB_ADULT: &[&str] = &[" ★\\o/★ ", "   |   ", "  / \\  "];
    pub const BLOB_ADULT_HAPPY: &[&str] = &["★(^o^)★", " \\|^|/ ", "  / \\  "];
    pub const BLOB_ADULT_WORK: &[&str] = &[" ★•_•★ ", "  <|>  ", "  / \\  "];
    pub const BLOB_ADULT_SLEEP: &[&str] = &[" ★-_-★ ", "  /|\\  ", " z   z "];

    // === CAT ===
    pub const CAT_BABY: &[&str] = &[" /\\_/\\ ", "( o.o )", " > ^ < "];
    pub const CAT_BABY_HAPPY: &[&str] = &[" /\\_/\\ ", "( ^w^ )", " > ~ < "];
    pub const CAT_BABY_WORK: &[&str] = &[" /\\_/\\ ", "( -.- )", " > n < "];
    pub const CAT_BABY_SLEEP: &[&str] = &[" /\\_/\\ ", "( -.- )", "  zZz  "];

    pub const CAT_TEEN: &[&str] = &["  /\\_/\\  ", " ( o.o ) ", "  />~<\\  "];
    pub const CAT_TEEN_HAPPY: &[&str] = &["  /\\_/\\  ", " ( ^o^ ) ", " ~/>~<\\~ "];
    pub const CAT_TEEN_WORK: &[&str] = &["  /\\_/\\  ", " ( -_- ) ", "  />o<\\  "];
    pub const CAT_TEEN_SLEEP: &[&str] = &["  /\\_/\\  ", " ( -.- ) ", " z/>~<\\z "];

    pub const CAT_ADULT: &[&str] = &[" ★/\\_/\\★", "  (o.o) ", "  />~<\\ "];
    pub const CAT_ADULT_HAPPY: &[&str] = &[" ★/\\_/\\★", "  (^ω^) ", " ★/>~<\\★"];
    pub const CAT_ADULT_WORK: &[&str] = &[" ★/\\_/\\★", "  (•_•) ", "  />o<\\ "];
    pub const CAT_ADULT_SLEEP: &[&str] = &[" ★/\\_/\\★", "  (-.-) ", " z/>~<\\z"];

    // === ROBOT ===
    pub const ROBOT_BABY: &[&str] = &[" [o_o] ", "  ]|[  ", "  d b  "];
    pub const ROBOT_BABY_HAPPY: &[&str] = &[" [^_^] ", " \\]|[/ ", "  d b  "];
    pub const ROBOT_BABY_WORK: &[&str] = &[" [0_0] ", "  ]|[  ", "  d|b  "];
    pub const ROBOT_BABY_SLEEP: &[&str] = &[" [-_-] ", "  ]|[  ", "  d b  "];

    pub const ROBOT_TEEN: &[&str] = &[" ┌[o_o]┐", "  ╠═╣  ", "  /I\\  "];
    pub const ROBOT_TEEN_HAPPY: &[&str] = &[" ┌[^_^]┐", " \\╠═╣/ ", "  /I\\  "];
    pub const ROBOT_TEEN_WORK: &[&str] = &[" ┌[0_0]┐", "  ╠═╣  ", "  /Y\\  "];
    pub const ROBOT_TEEN_SLEEP: &[&str] = &[" ┌[-_-]┐", "  ╠═╣  ", " z/I\\z "];

    pub const ROBOT_ADULT: &[&str] = &["★┌[o_o]┐★", "  ╠═╣  ", "  /Π\\  "];
    pub const ROBOT_ADULT_HAPPY: &[&str] = &["★┌[^▽^]┐★", " \\╠═╣/ ", "  /Π\\  "];
    pub const ROBOT_ADULT_WORK: &[&str] = &["★┌[◉_◉]┐★", "  ╠═╣  ", "  /Y\\  "];
    pub const ROBOT_ADULT_SLEEP: &[&str] = &["★┌[-_-]┐★", "  ╠═╣  ", " z/Π\\z "];

    // === GHOST ===
    pub const GHOST_BABY: &[&str] = &[" .---. ", "( o o )", " \\~~~/ "];
    pub const GHOST_BABY_HAPPY: &[&str] = &[" .---. ", "( ^o^ )", " \\^^^/ "];
    pub const GHOST_BABY_WORK: &[&str] = &[" .---. ", "( o_o )", " \\.../ "];
    pub const GHOST_BABY_SLEEP: &[&str] = &[" .---. ", "( -_- )", " \\zzz/ "];

    pub const GHOST_TEEN: &[&str] = &["  .--.  ", " ( oo ) ", " /|~~|\\ "];
    pub const GHOST_TEEN_HAPPY: &[&str] = &["  .--.  ", " ( ^^ ) ", "~/|~~|\\~"];
    pub const GHOST_TEEN_WORK: &[&str] = &["  .--.  ", " ( o_o) ", " /|..|\\"];
    pub const GHOST_TEEN_SLEEP: &[&str] = &["  .--.  ", " ( -.- ) ", "z/|~~|\\z"];

    pub const GHOST_ADULT: &[&str] = &[" ★.---.★", "  (o o) ", " ★\\~~~/★"];
    pub const GHOST_ADULT_HAPPY: &[&str] = &[" ★.---.★", "  (^o^) ", " ★\\^^^/★"];
    pub const GHOST_ADULT_WORK: &[&str] = &[" ★.---.★", "  (◉_◉) ", " ★\\_.~/★"];
    pub const GHOST_ADULT_SLEEP: &[&str] = &[" ★.---.★", "  (-_-) ", "z★\\~~~/★"];

    pub fn get_art(pet_type: PetType, stage: u32, mood: super::PetMood, frame: usize) -> &'static [&'static str] {
        use super::PetMood;

        // Egg stage - all pets
        if stage == 1 {
            let wobble = frame % 4 == 0;
            return match pet_type {
                PetType::Blob => if wobble { EGG_BLOB_WOBBLE } else { EGG_BLOB },
                PetType::Cat => if wobble { EGG_CAT_WOBBLE } else { EGG_CAT },
                PetType::Robot => if wobble { EGG_ROBOT_WOBBLE } else { EGG_ROBOT },
                PetType::Ghost => if wobble { EGG_GHOST_WOBBLE } else { EGG_GHOST },
            };
        }

        match pet_type {
            PetType::Blob => match stage {
                2 => match mood {
                    PetMood::Working => BLOB_BABY_WORK,
                    PetMood::Happy => BLOB_BABY_HAPPY,
                    PetMood::Resting => BLOB_BABY_SLEEP,
                    PetMood::Idle => BLOB_BABY,
                },
                3 => match mood {
                    PetMood::Working => BLOB_TEEN_WORK,
                    PetMood::Happy => BLOB_TEEN_HAPPY,
                    PetMood::Resting => BLOB_TEEN_SLEEP,
                    PetMood::Idle => BLOB_TEEN,
                },
                _ => match mood {
                    PetMood::Working => BLOB_ADULT_WORK,
                    PetMood::Happy => BLOB_ADULT_HAPPY,
                    PetMood::Resting => BLOB_ADULT_SLEEP,
                    PetMood::Idle => BLOB_ADULT,
                },
            },
            PetType::Cat => match stage {
                2 => match mood {
                    PetMood::Working => CAT_BABY_WORK,
                    PetMood::Happy => CAT_BABY_HAPPY,
                    PetMood::Resting => CAT_BABY_SLEEP,
                    PetMood::Idle => CAT_BABY,
                },
                3 => match mood {
                    PetMood::Working => CAT_TEEN_WORK,
                    PetMood::Happy => CAT_TEEN_HAPPY,
                    PetMood::Resting => CAT_TEEN_SLEEP,
                    PetMood::Idle => CAT_TEEN,
                },
                _ => match mood {
                    PetMood::Working => CAT_ADULT_WORK,
                    PetMood::Happy => CAT_ADULT_HAPPY,
                    PetMood::Resting => CAT_ADULT_SLEEP,
                    PetMood::Idle => CAT_ADULT,
                },
            },
            PetType::Robot => match stage {
                2 => match mood {
                    PetMood::Working => ROBOT_BABY_WORK,
                    PetMood::Happy => ROBOT_BABY_HAPPY,
                    PetMood::Resting => ROBOT_BABY_SLEEP,
                    PetMood::Idle => ROBOT_BABY,
                },
                3 => match mood {
                    PetMood::Working => ROBOT_TEEN_WORK,
                    PetMood::Happy => ROBOT_TEEN_HAPPY,
                    PetMood::Resting => ROBOT_TEEN_SLEEP,
                    PetMood::Idle => ROBOT_TEEN,
                },
                _ => match mood {
                    PetMood::Working => ROBOT_ADULT_WORK,
                    PetMood::Happy => ROBOT_ADULT_HAPPY,
                    PetMood::Resting => ROBOT_ADULT_SLEEP,
                    PetMood::Idle => ROBOT_ADULT,
                },
            },
            PetType::Ghost => match stage {
                2 => match mood {
                    PetMood::Working => GHOST_BABY_WORK,
                    PetMood::Happy => GHOST_BABY_HAPPY,
                    PetMood::Resting => GHOST_BABY_SLEEP,
                    PetMood::Idle => GHOST_BABY,
                },
                3 => match mood {
                    PetMood::Working => GHOST_TEEN_WORK,
                    PetMood::Happy => GHOST_TEEN_HAPPY,
                    PetMood::Resting => GHOST_TEEN_SLEEP,
                    PetMood::Idle => GHOST_TEEN,
                },
                _ => match mood {
                    PetMood::Working => GHOST_ADULT_WORK,
                    PetMood::Happy => GHOST_ADULT_HAPPY,
                    PetMood::Resting => GHOST_ADULT_SLEEP,
                    PetMood::Idle => GHOST_ADULT,
                },
            },
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    Timer,
    Pet,
    Stats,
    Debug,
}

#[derive(PartialEq, Clone, Copy)]
enum PomodoroState {
    Work,
    Break,
    Paused,
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
enum PetMood {
    Idle,
    Working,
    Happy,
    Resting,
}

#[derive(Serialize, Deserialize)]
struct GameData {
    // XP and leveling
    xp: u32,
    level: u32,
    // Stats
    total_sessions: u32,
    total_focus_mins: u32,
    last_session_date: Option<NaiveDate>,
    streak_days: u32,
    today_sessions: u32,
    today_date: Option<NaiveDate>,
    // Pet
    pet_name: String,
    pet_type: PetType,
    mood: PetMood,
}

impl Default for GameData {
    fn default() -> Self {
        let pet_type = PetType::random();
        Self {
            xp: 0,
            level: 1,
            total_sessions: 0,
            total_focus_mins: 0,
            last_session_date: None,
            streak_days: 0,
            today_sessions: 0,
            today_date: Some(Local::now().date_naive()),
            pet_name: "Tomo".to_string(),
            pet_type,
            mood: PetMood::Idle,
        }
    }
}

impl GameData {
    fn data_path() -> PathBuf {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tamadoro");
        fs::create_dir_all(&data_dir).ok();
        data_dir.join("save.json")
    }

    fn load() -> Self {
        let path = Self::data_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(mut game) = serde_json::from_str::<GameData>(&data) {
                let today = Local::now().date_naive();
                if game.today_date != Some(today) {
                    game.today_sessions = 0;
                    game.today_date = Some(today);
                }
                return game;
            }
        }
        GameData::default()
    }

    fn save(&self) {
        let path = Self::data_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            fs::write(path, data).ok();
        }
    }

    fn xp_for_level(level: u32) -> u32 {
        // XP needed: 100, 150, 225, 337, ...
        (100.0 * 1.5_f64.powi((level - 1) as i32)) as u32
    }

    fn xp_to_next_level(&self) -> u32 {
        Self::xp_for_level(self.level)
    }

    fn add_xp(&mut self, amount: u32) {
        self.xp += amount;
        while self.xp >= self.xp_to_next_level() {
            self.xp -= self.xp_to_next_level();
            self.level += 1;
        }
    }

    fn record_session(&mut self) {
        let today = Local::now().date_naive();

        // Update streak
        if let Some(last_date) = self.last_session_date {
            let days_diff = (today - last_date).num_days();
            if days_diff == 1 {
                self.streak_days += 1;
            } else if days_diff > 1 {
                self.streak_days = 1;
            }
        } else {
            self.streak_days = 1;
        }

        self.last_session_date = Some(today);
        self.total_sessions += 1;
        self.total_focus_mins += 25;

        if self.today_date != Some(today) {
            self.today_sessions = 0;
            self.today_date = Some(today);
        }
        self.today_sessions += 1;

        // Award XP with streak bonus
        let base_xp = 25;
        let streak_bonus = (self.streak_days.min(7) * 5) as u32;
        self.add_xp(base_xp + streak_bonus);

        self.mood = PetMood::Happy;
        self.save();
    }

    fn evolution_stage(&self) -> u32 {
        if self.level < 5 {
            1 // Egg
        } else if self.level < 15 {
            2 // Baby
        } else if self.level < 30 {
            3 // Teen
        } else {
            4 // Adult
        }
    }

    fn get_pet_art(&self, frame: usize) -> &'static [&'static str] {
        pet::get_art(self.pet_type, self.evolution_stage(), self.mood, frame)
    }

    fn stage_name(&self) -> &'static str {
        match self.evolution_stage() {
            1 => "Egg",
            2 => "Baby",
            3 => "Teen",
            _ => "Master",
        }
    }
}

struct App {
    mode: Mode,
    pomo_state: PomodoroState,
    pomo_remaining: Duration,
    pomo_total: Duration,
    pomo_sessions: u32,
    last_tick: Instant,
    game: GameData,
    frame: usize,
    message: Option<(String, Instant)>,
}

impl App {
    fn new() -> Self {
        let work_duration = Duration::from_secs(25 * 60);

        App {
            mode: Mode::Timer,
            pomo_state: PomodoroState::Paused,
            pomo_remaining: work_duration,
            pomo_total: work_duration,
            pomo_sessions: 0,
            last_tick: Instant::now(),
            game: GameData::load(),
            frame: 0,
            message: None,
        }
    }

    fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);

        // Clear old messages
        if let Some((_, time)) = &self.message {
            if time.elapsed() > Duration::from_secs(3) {
                self.message = None;
            }
        }

        if self.pomo_state == PomodoroState::Work || self.pomo_state == PomodoroState::Break {
            let elapsed = self.last_tick.elapsed();
            self.last_tick = Instant::now();

            if self.pomo_remaining > elapsed {
                self.pomo_remaining -= elapsed;
            } else {
                if self.pomo_state == PomodoroState::Work {
                    self.pomo_sessions += 1;
                    let old_level = self.game.level;
                    self.game.record_session();

                    if self.game.level > old_level {
                        self.message = Some((
                            format!("LEVEL UP! Lv.{}", self.game.level),
                            Instant::now(),
                        ));
                        if self.game.evolution_stage()
                            > (GameData {
                                level: old_level,
                                ..Default::default()
                            })
                            .evolution_stage()
                        {
                            self.message = Some((
                                format!("{} evolved!", self.game.pet_name),
                                Instant::now(),
                            ));
                        }
                    } else {
                        let messages = [
                            "Great work!",
                            "Amazing focus!",
                            "You're on fire!",
                            "Keep it up!",
                            "Fantastic!",
                        ];
                        let msg = messages[rand::thread_rng().gen_range(0..messages.len())];
                        self.message = Some((msg.to_string(), Instant::now()));
                    }

                    self.pomo_state = PomodoroState::Break;
                    self.pomo_total = Duration::from_secs(5 * 60);
                    self.pomo_remaining = self.pomo_total;
                    self.game.mood = PetMood::Resting;
                } else {
                    self.pomo_state = PomodoroState::Paused;
                    self.pomo_total = Duration::from_secs(25 * 60);
                    self.pomo_remaining = self.pomo_total;
                    self.game.mood = PetMood::Idle;
                    self.message = Some(("Break over! Ready?".to_string(), Instant::now()));
                }
            }
        }
    }

    fn toggle_pomo(&mut self) {
        match self.pomo_state {
            PomodoroState::Paused => {
                self.pomo_state = PomodoroState::Work;
                self.last_tick = Instant::now();
                self.game.mood = PetMood::Working;
                self.game.save();
            }
            _ => {
                self.pomo_state = PomodoroState::Paused;
                self.game.mood = PetMood::Idle;
                self.game.save();
            }
        }
    }

    fn reset_pomo(&mut self) {
        self.pomo_state = PomodoroState::Paused;
        self.pomo_total = Duration::from_secs(25 * 60);
        self.pomo_remaining = self.pomo_total;
        self.game.mood = PetMood::Idle;
        self.game.save();
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Tab => {
                            app.mode = match app.mode {
                                Mode::Timer => Mode::Pet,
                                Mode::Pet => Mode::Stats,
                                Mode::Stats => Mode::Debug,
                                Mode::Debug => Mode::Timer,
                            };
                        }
                        KeyCode::Char(' ') => {
                            app.toggle_pomo();
                        }
                        KeyCode::Char('r') if app.mode == Mode::Timer => {
                            app.reset_pomo();
                        }
                        // Debug controls
                        KeyCode::Char('1') if app.mode == Mode::Debug => {
                            app.game.add_xp(50);
                            app.game.save();
                            app.message = Some(("+50 XP".to_string(), Instant::now()));
                        }
                        KeyCode::Char('2') if app.mode == Mode::Debug => {
                            app.game.add_xp(500);
                            app.game.save();
                            app.message = Some(("+500 XP".to_string(), Instant::now()));
                        }
                        KeyCode::Char('3') if app.mode == Mode::Debug => {
                            app.game.xp = 0;
                            app.game.level += 1;
                            app.game.save();
                            app.message = Some((format!("Level -> {}", app.game.level), Instant::now()));
                        }
                        KeyCode::Char('4') if app.mode == Mode::Debug => {
                            // Jump to next evolution stage
                            let next_level = match app.game.evolution_stage() {
                                1 => 5,
                                2 => 15,
                                3 => 30,
                                _ => app.game.level + 10,
                            };
                            app.game.level = next_level;
                            app.game.xp = 0;
                            app.game.save();
                            app.message = Some((format!("Evolved to {}!", app.game.stage_name()), Instant::now()));
                        }
                        KeyCode::Char('5') if app.mode == Mode::Debug => {
                            app.game.streak_days = (app.game.streak_days + 1) % 15;
                            app.game.save();
                            app.message = Some((format!("Streak -> {}", app.game.streak_days), Instant::now()));
                        }
                        KeyCode::Char('6') if app.mode == Mode::Debug => {
                            app.game.pet_type = match app.game.pet_type {
                                PetType::Blob => PetType::Cat,
                                PetType::Cat => PetType::Robot,
                                PetType::Robot => PetType::Ghost,
                                PetType::Ghost => PetType::Blob,
                            };
                            app.game.save();
                            app.message = Some((format!("Pet -> {}", app.game.pet_type.name()), Instant::now()));
                        }
                        KeyCode::Char('0') if app.mode == Mode::Debug => {
                            app.game = GameData::default();
                            app.game.save();
                            app.pomo_sessions = 0;
                            app.message = Some(("Reset all data!".to_string(), Instant::now()));
                        }
                        _ => {}
                    }
                }
            }
        }

        app.tick();
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(match app.mode {
            Mode::Timer => colors::MAGENTA,
            Mode::Pet => colors::CYAN,
            Mode::Stats => colors::GREEN,
            Mode::Debug => colors::RED,
        }))
        .style(Style::default().bg(colors::BG));

    f.render_widget(block, area);

    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tabs
            Constraint::Length(1), // Spacer
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Message/Help
        ])
        .split(inner);

    // Tabs
    let tabs = Line::from(vec![
        Span::styled(
            "TIMER",
            if app.mode == Mode::Timer {
                Style::default().fg(colors::BG).bg(colors::MAGENTA).bold()
            } else {
                Style::default().fg(colors::COMMENT)
            },
        ),
        Span::raw(" "),
        Span::styled(
            "PET",
            if app.mode == Mode::Pet {
                Style::default().fg(colors::BG).bg(colors::CYAN).bold()
            } else {
                Style::default().fg(colors::COMMENT)
            },
        ),
        Span::raw(" "),
        Span::styled(
            "STATS",
            if app.mode == Mode::Stats {
                Style::default().fg(colors::BG).bg(colors::GREEN).bold()
            } else {
                Style::default().fg(colors::COMMENT)
            },
        ),
        Span::raw(" "),
        Span::styled(
            "DEBUG",
            if app.mode == Mode::Debug {
                Style::default().fg(colors::BG).bg(colors::RED).bold()
            } else {
                Style::default().fg(colors::COMMENT)
            },
        ),
    ]);
    f.render_widget(Paragraph::new(tabs).alignment(Alignment::Center), chunks[0]);

    match app.mode {
        Mode::Timer => render_timer(f, chunks[2], app),
        Mode::Pet => render_pet(f, chunks[2], app),
        Mode::Stats => render_stats(f, chunks[2], app),
        Mode::Debug => render_debug(f, chunks[2], app),
    }

    // Message or help
    let bottom_text = if let Some((msg, _)) = &app.message {
        Line::from(Span::styled(msg, Style::default().fg(colors::YELLOW).bold()))
    } else {
        Line::from(vec![
            Span::styled("SPC", Style::default().fg(colors::YELLOW)),
            Span::styled(" go ", Style::default().fg(colors::COMMENT)),
            Span::styled("TAB", Style::default().fg(colors::YELLOW)),
            Span::styled(" tab ", Style::default().fg(colors::COMMENT)),
            Span::styled("q", Style::default().fg(colors::YELLOW)),
            Span::styled(" quit", Style::default().fg(colors::COMMENT)),
        ])
    };
    f.render_widget(
        Paragraph::new(bottom_text).alignment(Alignment::Center),
        chunks[3],
    );
}

fn render_timer(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Pet preview
            Constraint::Length(1), // State
            Constraint::Length(2), // Timer
            Constraint::Length(1), // Progress
            Constraint::Length(1), // XP bar
            Constraint::Min(0),    // Spacer
        ])
        .split(area);

    // Mini pet
    let pet_art = app.game.get_pet_art(app.frame / 2);
    let pet_text: Vec<Line> = pet_art
        .iter()
        .map(|line| {
            Line::from(Span::styled(
                *line,
                Style::default().fg(if app.pomo_state == PomodoroState::Work {
                    colors::RED
                } else if app.pomo_state == PomodoroState::Break {
                    colors::GREEN
                } else {
                    colors::CYAN
                }),
            ))
        })
        .collect();
    f.render_widget(
        Paragraph::new(pet_text).alignment(Alignment::Center),
        chunks[0],
    );

    // State
    let (state_text, state_color) = match app.pomo_state {
        PomodoroState::Work => ("FOCUSING", colors::RED),
        PomodoroState::Break => ("RESTING", colors::GREEN),
        PomodoroState::Paused => ("READY", colors::COMMENT),
    };
    f.render_widget(
        Paragraph::new(state_text)
            .style(Style::default().fg(state_color).bold())
            .alignment(Alignment::Center),
        chunks[1],
    );

    // Timer
    let mins = app.pomo_remaining.as_secs() / 60;
    let secs = app.pomo_remaining.as_secs() % 60;
    f.render_widget(
        Paragraph::new(format!("{:02}:{:02}", mins, secs))
            .style(Style::default().fg(colors::FG).bold())
            .alignment(Alignment::Center),
        chunks[2],
    );

    // Progress
    let progress = if app.pomo_total.as_secs() > 0 {
        1.0 - (app.pomo_remaining.as_secs_f64() / app.pomo_total.as_secs_f64())
    } else {
        0.0
    };
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(match app.pomo_state {
                PomodoroState::Work => colors::RED,
                PomodoroState::Break => colors::GREEN,
                PomodoroState::Paused => colors::COMMENT,
            }))
            .ratio(progress)
            .label(""),
        chunks[3],
    );

    // XP bar
    let xp_progress = app.game.xp as f64 / app.game.xp_to_next_level() as f64;
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(colors::YELLOW))
            .ratio(xp_progress)
            .label(format!("Lv.{}", app.game.level)),
        chunks[4],
    );
}

fn render_pet(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Name
            Constraint::Length(5), // Pet art
            Constraint::Length(1), // Stage
            Constraint::Length(1), // Level
            Constraint::Length(1), // XP
            Constraint::Length(1), // XP bar
            Constraint::Min(0),    // Spacer
        ])
        .split(area);

    // Name
    f.render_widget(
        Paragraph::new(format!("~ {} ~", app.game.pet_name))
            .style(Style::default().fg(colors::CYAN).bold())
            .alignment(Alignment::Center),
        chunks[0],
    );

    // Pet art
    let pet_art = app.game.get_pet_art(app.frame / 2);
    let pet_text: Vec<Line> = pet_art
        .iter()
        .map(|line| {
            Line::from(Span::styled(
                *line,
                Style::default().fg(match app.game.mood {
                    PetMood::Working => colors::RED,
                    PetMood::Happy => colors::YELLOW,
                    PetMood::Resting => colors::GREEN,
                    PetMood::Idle => colors::CYAN,
                }),
            ))
        })
        .collect();
    f.render_widget(
        Paragraph::new(pet_text).alignment(Alignment::Center),
        chunks[1],
    );

    // Stage + Type
    f.render_widget(
        Paragraph::new(format!("{} {}", app.game.pet_type.name(), app.game.stage_name()))
            .style(Style::default().fg(colors::MAGENTA))
            .alignment(Alignment::Center),
        chunks[2],
    );

    // Level
    f.render_widget(
        Paragraph::new(format!("Level {}", app.game.level))
            .style(Style::default().fg(colors::FG).bold())
            .alignment(Alignment::Center),
        chunks[3],
    );

    // XP
    f.render_widget(
        Paragraph::new(format!(
            "XP: {}/{}",
            app.game.xp,
            app.game.xp_to_next_level()
        ))
        .style(Style::default().fg(colors::YELLOW))
        .alignment(Alignment::Center),
        chunks[4],
    );

    // XP progress bar
    let xp_progress = app.game.xp as f64 / app.game.xp_to_next_level() as f64;
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(colors::YELLOW))
            .ratio(xp_progress)
            .label(""),
        chunks[5],
    );
}

fn render_stats(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Today
            Constraint::Length(1), // Total sessions
            Constraint::Length(1), // Total time
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Streak
            Constraint::Length(1), // Bonus
            Constraint::Min(0),    // Spacer
        ])
        .split(area);

    f.render_widget(
        Paragraph::new("Progress")
            .style(Style::default().fg(colors::GREEN).bold())
            .alignment(Alignment::Center),
        chunks[0],
    );

    f.render_widget(
        Paragraph::new(format!("Today: {} sessions", app.game.today_sessions))
            .style(Style::default().fg(colors::FG))
            .alignment(Alignment::Center),
        chunks[2],
    );

    f.render_widget(
        Paragraph::new(format!("Total: {}", app.game.total_sessions))
            .style(Style::default().fg(colors::FG))
            .alignment(Alignment::Center),
        chunks[3],
    );

    let hours = app.game.total_focus_mins / 60;
    let mins = app.game.total_focus_mins % 60;
    f.render_widget(
        Paragraph::new(format!("Time: {}h {}m", hours, mins))
            .style(Style::default().fg(colors::COMMENT))
            .alignment(Alignment::Center),
        chunks[4],
    );

    f.render_widget(
        Paragraph::new(format!("Streak: {} days", app.game.streak_days))
            .style(Style::default().fg(colors::YELLOW).bold())
            .alignment(Alignment::Center),
        chunks[6],
    );

    let bonus = (app.game.streak_days.min(7) * 5) as u32;
    f.render_widget(
        Paragraph::new(format!("+{} XP bonus", bonus))
            .style(Style::default().fg(colors::MAGENTA))
            .alignment(Alignment::Center),
        chunks[7],
    );
}

fn render_debug(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Current state
            Constraint::Length(1), // Level/XP
            Constraint::Length(1), // Stage
            Constraint::Length(1), // Streak
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Controls header
            Constraint::Length(1), // Control 1
            Constraint::Length(1), // Control 2
            Constraint::Length(1), // Control 3
            Constraint::Length(1), // Control 4
            Constraint::Length(1), // Control 5
            Constraint::Min(0),    // Spacer
        ])
        .split(area);

    f.render_widget(
        Paragraph::new("Debug Panel")
            .style(Style::default().fg(colors::RED).bold())
            .alignment(Alignment::Center),
        chunks[0],
    );

    f.render_widget(
        Paragraph::new(format!(
            "Lv.{} | XP: {}/{}",
            app.game.level,
            app.game.xp,
            app.game.xp_to_next_level()
        ))
        .style(Style::default().fg(colors::FG))
        .alignment(Alignment::Center),
        chunks[2],
    );

    f.render_widget(
        Paragraph::new(format!("{} {} (Stg {})", app.game.pet_type.name(), app.game.stage_name(), app.game.evolution_stage()))
            .style(Style::default().fg(colors::CYAN))
            .alignment(Alignment::Center),
        chunks[3],
    );

    f.render_widget(
        Paragraph::new(format!("Streak: {} days", app.game.streak_days))
            .style(Style::default().fg(colors::YELLOW))
            .alignment(Alignment::Center),
        chunks[4],
    );

    f.render_widget(
        Paragraph::new(format!("Sessions: {}", app.game.total_sessions))
            .style(Style::default().fg(colors::COMMENT))
            .alignment(Alignment::Center),
        chunks[5],
    );

    f.render_widget(
        Paragraph::new("─ Controls ─")
            .style(Style::default().fg(colors::COMMENT))
            .alignment(Alignment::Center),
        chunks[7],
    );

    f.render_widget(
        Paragraph::new("1: +50 XP  2: +500 XP")
            .style(Style::default().fg(colors::GREEN))
            .alignment(Alignment::Center),
        chunks[8],
    );

    f.render_widget(
        Paragraph::new("3: +1 Lv   4: Evolve")
            .style(Style::default().fg(colors::MAGENTA))
            .alignment(Alignment::Center),
        chunks[9],
    );

    f.render_widget(
        Paragraph::new("5: +Streak 6: Pet")
            .style(Style::default().fg(colors::CYAN))
            .alignment(Alignment::Center),
        chunks[10],
    );

    f.render_widget(
        Paragraph::new("0: RESET ALL")
            .style(Style::default().fg(colors::RED))
            .alignment(Alignment::Center),
        chunks[11],
    );
}
