use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

static TEST_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_test_mode(on: bool) {
    TEST_MODE.store(on, Ordering::Relaxed);
}

pub fn is_test_mode() -> bool {
    TEST_MODE.load(Ordering::Relaxed)
}

use chrono::{Local, NaiveDate, Timelike};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::pets;

pub const SAVE_VERSION: u32 = 2;

// Pet types
#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum PetType {
    Blob,
    Cat,
    Robot,
    Ghost,
}

impl PetType {
    pub fn random() -> Self {
        match rand::thread_rng().gen_range(0..4) {
            0 => PetType::Blob,
            1 => PetType::Cat,
            2 => PetType::Robot,
            _ => PetType::Ghost,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PetType::Blob => "Blob",
            PetType::Cat => "Cat",
            PetType::Robot => "Robot",
            PetType::Ghost => "Ghost",
        }
    }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum PetMood {
    Idle,
    Working,
    Happy,
    Resting,
}

/// Per-pet data. Will be snapshotted into the Hall of Fame on graduation.
#[derive(Serialize, Deserialize, Clone)]
pub struct Pet {
    pub name: String,
    pub pet_type: PetType,
    pub mood: PetMood,
    pub xp: u32,
    pub level: u32,
    pub food: u32,
    #[serde(default)]
    pub last_food_check: Option<i64>,
    #[serde(default)]
    pub hunger_zero_since: Option<i64>,
    #[serde(default)]
    pub is_dead: bool,
    // Lifetime tracking (used for personality derivation at graduation).
    #[serde(default)]
    pub lifetime_sessions: u32,
    #[serde(default)]
    pub lifetime_focus_mins: u32,
    #[serde(default)]
    pub times_hungry: u32,
    #[serde(default)]
    pub times_fed: u32,
    #[serde(default)]
    pub born_at: i64,
    #[serde(default)]
    pub victory_lap_sessions: u32,
}

impl Pet {
    pub fn random_name() -> String {
        const NAMES: &[&str] = &[
            "Tomo", "Pip", "Zuki", "Mochi", "Nori", "Kiwi", "Bubu", "Yuki",
            "Taro", "Momo", "Suki", "Rin", "Kai", "Tako", "Miso", "Pon",
            "Luma", "Eko", "Zen", "Ari", "Biscuit", "Nimbus", "Pebble",
        ];
        NAMES[rand::thread_rng().gen_range(0..NAMES.len())].to_string()
    }

    pub fn new_random() -> Self {
        let now = Local::now().timestamp();
        Self {
            name: Self::random_name(),
            pet_type: PetType::random(),
            mood: PetMood::Idle,
            xp: 0,
            level: 1,
            food: 100,
            last_food_check: Some(now),
            hunger_zero_since: None,
            is_dead: false,
            lifetime_sessions: 0,
            lifetime_focus_mins: 0,
            times_hungry: 0,
            times_fed: 0,
            born_at: now,
            victory_lap_sessions: 0,
        }
    }

    pub fn xp_for_level(level: u32) -> u32 {
        // Linear curve tuned so Master (lvl 6) lands around 45 sessions
        // (~18h focus) at the current ~30 XP/session award rate.
        // L1→2: 160, L2→3: 220, ... L5→6: 400. Cumulative to L6 ≈ 1400 XP.
        100 + 60 * level
    }

    pub fn xp_to_next_level(&self) -> u32 {
        Self::xp_for_level(self.level)
    }

    pub fn add_xp(&mut self, amount: u32) {
        self.xp += amount;
        while self.xp >= self.xp_to_next_level() {
            self.xp -= self.xp_to_next_level();
            self.level += 1;
        }
    }

    pub fn evolution_stage_for_level(level: u32) -> u32 {
        if level < 2 {
            1 // Egg
        } else if level < 4 {
            2 // Baby
        } else if level < 6 {
            3 // Teen
        } else {
            4 // Master
        }
    }

    pub fn evolution_stage(&self) -> u32 {
        Self::evolution_stage_for_level(self.level)
    }

    pub fn stage_name(&self) -> &'static str {
        match self.evolution_stage() {
            1 => "Egg",
            2 => "Baby",
            3 => "Teen",
            _ => "Master",
        }
    }

    pub fn get_art(&self, frame: usize) -> &'static [&'static str] {
        if self.is_dead {
            pets::get_dead_art(self.pet_type)
        } else {
            pets::get_art(self.pet_type, self.evolution_stage(), self.mood, frame)
        }
    }

    pub fn feed(&mut self, amount: u32) {
        self.food = (self.food + amount).min(100);
        if self.food > 0 {
            self.hunger_zero_since = None;
        }
        self.times_fed = self.times_fed.saturating_add(1);
    }

    pub fn hunger_cry(&self) -> &'static str {
        match self.pet_type {
            PetType::Blob => "I'm so hungry... please help me! I need food! 😢",
            PetType::Cat => "Meow... *weak* ...please... food... I'm starving... 🐱",
            PetType::Robot => "CRITICAL: Energy depleted... shutting down... need fuel... 🤖",
            PetType::Ghost => "I'm fading away... so hungry... please don't let me disappear... 👻",
        }
    }

    pub fn death_cry(&self) -> &'static str {
        match self.pet_type {
            PetType::Blob => "Goodbye... I waited but you never came... 💔",
            PetType::Cat => "Meow... *silence* ...goodbye friend... 🐱💔",
            PetType::Robot => "SYSTEM FAILURE... memory banks... fading... goodbye... 🤖💔",
            PetType::Ghost => "I'm disappearing forever... I'll miss you... 👻💔",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Outcome {
    Graduated,
    Memorial,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HallOfFameEntry {
    pub pet: Pet,
    pub graduated_at: i64,
    #[serde(default = "default_outcome")]
    pub outcome: Outcome,
}

fn default_outcome() -> Outcome {
    Outcome::Graduated
}

#[derive(Serialize, Deserialize)]
pub struct GameData {
    #[serde(default)]
    pub save_version: u32,

    // Global stats — carry across pets.
    pub total_sessions: u32,
    pub total_focus_mins: u32,
    pub last_session_date: Option<NaiveDate>,
    pub streak_days: u32,
    pub today_sessions: u32,
    pub today_date: Option<NaiveDate>,

    // Current pet. None means the Hatchery should show (Phase 4).
    #[serde(default)]
    pub current: Option<Pet>,

    #[serde(default)]
    pub hall_of_fame: Vec<HallOfFameEntry>,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            save_version: SAVE_VERSION,
            total_sessions: 0,
            total_focus_mins: 0,
            last_session_date: None,
            streak_days: 0,
            today_sessions: 0,
            today_date: Some(Local::now().date_naive()),
            current: Some(Pet::new_random()),
            hall_of_fame: Vec::new(),
        }
    }
}

/// Legacy (v1) flat save format. Kept only for migration.
#[derive(Deserialize)]
struct LegacyGameData {
    xp: u32,
    level: u32,
    total_sessions: u32,
    total_focus_mins: u32,
    last_session_date: Option<NaiveDate>,
    streak_days: u32,
    today_sessions: u32,
    today_date: Option<NaiveDate>,
    pet_name: String,
    pet_type: PetType,
    mood: PetMood,
    food: u32,
    #[serde(default)]
    last_food_check: Option<i64>,
    #[serde(default)]
    hunger_zero_since: Option<i64>,
    #[serde(default)]
    is_dead: bool,
}

impl LegacyGameData {
    fn migrate(self) -> GameData {
        let now = Local::now().timestamp();
        let pet = Pet {
            name: self.pet_name,
            pet_type: self.pet_type,
            mood: self.mood,
            xp: self.xp,
            level: self.level,
            food: self.food,
            last_food_check: self.last_food_check.or(Some(now)),
            hunger_zero_since: self.hunger_zero_since,
            is_dead: self.is_dead,
            lifetime_sessions: self.total_sessions, // best-effort backfill
            lifetime_focus_mins: self.total_focus_mins,
            times_hungry: 0,
            times_fed: 0,
            born_at: now, // unknown; use migration time as birth
            victory_lap_sessions: 0,
        };
        GameData {
            save_version: SAVE_VERSION,
            total_sessions: self.total_sessions,
            total_focus_mins: self.total_focus_mins,
            last_session_date: self.last_session_date,
            streak_days: self.streak_days,
            today_sessions: self.today_sessions,
            today_date: self.today_date,
            current: Some(pet),
            hall_of_fame: Vec::new(),
        }
    }
}

impl GameData {
    fn data_dir() -> PathBuf {
        let dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tamadoro");
        fs::create_dir_all(&dir).ok();
        dir
    }

    pub fn real_save_path() -> PathBuf {
        Self::data_dir().join("save.json")
    }

    pub fn test_save_path() -> PathBuf {
        Self::data_dir().join("save.test.json")
    }

    fn data_path() -> PathBuf {
        if is_test_mode() {
            Self::test_save_path()
        } else {
            Self::real_save_path()
        }
    }

    /// Copy the real save to the test save path so --test runs on a clone.
    pub fn seed_test_save() {
        let real = Self::real_save_path();
        let test = Self::test_save_path();
        if real.exists() {
            let _ = fs::copy(&real, &test);
        } else {
            // No real save yet — make sure we start from a clean slate.
            let _ = fs::remove_file(&test);
        }
    }

    fn backup_legacy(path: &PathBuf) {
        let Some(parent) = path.parent() else { return };
        let stamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let backup = parent.join(format!("save.json.legacy-v1.{}", stamp));
        let _ = fs::copy(path, backup);
    }

    fn rollover_day(&mut self) {
        let today = Local::now().date_naive();
        if self.today_date != Some(today) {
            self.today_sessions = 0;
            self.today_date = Some(today);
        }
    }

    pub fn load() -> Self {
        let path = Self::data_path();
        let Ok(data) = fs::read_to_string(&path) else {
            return GameData::default();
        };

        // Peek at save_version to decide which schema to use.
        let version = serde_json::from_str::<serde_json::Value>(&data)
            .ok()
            .and_then(|v| v.get("save_version").and_then(|sv| sv.as_u64()))
            .unwrap_or(0) as u32;

        if version >= 2 {
            if let Ok(mut game) = serde_json::from_str::<GameData>(&data) {
                game.rollover_day();
                return game;
            }
        }

        // Legacy (v1) migration path.
        if let Ok(legacy) = serde_json::from_str::<LegacyGameData>(&data) {
            Self::backup_legacy(&path);
            let mut game = legacy.migrate();
            game.rollover_day();
            game.save();
            return game;
        }

        GameData::default()
    }

    pub fn save(&self) {
        let path = Self::data_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            fs::write(path, data).ok();
        }
    }

    /// Convenience accessor: panics if there is no current pet.
    /// Phase 1 callers are guaranteed a current pet post-load; Phase 4 will
    /// thread Option handling through the UI instead.
    pub fn pet(&self) -> &Pet {
        self.current
            .as_ref()
            .expect("GameData::pet() called with no current pet")
    }

    pub fn pet_mut(&mut self) -> &mut Pet {
        self.current
            .as_mut()
            .expect("GameData::pet_mut() called with no current pet")
    }

    pub fn record_session(&mut self) {
        let today = Local::now().date_naive();

        // Update streak (global).
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

        // Award XP with streak bonus to the current pet.
        let base_xp = 25;
        let streak_bonus = (self.streak_days.min(7) * 5) as u32;
        let pet = self.pet_mut();
        pet.add_xp(base_xp + streak_bonus);
        pet.lifetime_sessions = pet.lifetime_sessions.saturating_add(1);
        pet.lifetime_focus_mins = pet.lifetime_focus_mins.saturating_add(25);
        pet.feed(25);
        pet.mood = PetMood::Happy;

        // Victory lap: sessions completed while already at Master stage.
        if pet.evolution_stage() >= 4 {
            pet.victory_lap_sessions = pet.victory_lap_sessions.saturating_add(1);
        }

        self.save();
    }

    pub const VICTORY_LAP_GOAL: u32 = 10;

    /// Graduate the current pet if it has earned its cap and gown.
    /// Returns true if graduation happened (caller should show a message
    /// and, for now, auto-spawn a replacement pet).
    pub fn try_graduate(&mut self) -> bool {
        let ready = self
            .current
            .as_ref()
            .map(|p| {
                !p.is_dead
                    && p.evolution_stage() >= 4
                    && p.victory_lap_sessions >= Self::VICTORY_LAP_GOAL
            })
            .unwrap_or(false);
        if !ready {
            return false;
        }
        let pet = self.current.take().unwrap();
        self.hall_of_fame.push(HallOfFameEntry {
            pet,
            graduated_at: Local::now().timestamp(),
            outcome: Outcome::Graduated,
        });
        // current stays None — Hatchery takes over until user picks a new pet.
        self.save();
        true
    }

    fn is_daytime() -> bool {
        let hour = Local::now().hour();
        hour >= 6 && hour < 22 // 6am to 10pm
    }

    fn send_notification(title: &str, message: &str) {
        let script = format!(
            "display notification \"{}\" with title \"{}\"",
            message.replace('"', "\\\""),
            title.replace('"', "\\\"")
        );
        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .ok();
    }

    pub fn update_food(&mut self) {
        // No current pet → nothing to do (Hatchery state in later phases).
        if self.current.is_none() {
            return;
        }
        // Don't update if already dead.
        if self.pet().is_dead {
            return;
        }

        let now = Local::now().timestamp();
        let is_daytime = Self::is_daytime();

        let pet = self.pet_mut();

        // Initialize if not set.
        if pet.last_food_check.is_none() {
            pet.last_food_check = Some(now);
            return;
        }

        let last_check = pet.last_food_check.unwrap();
        let elapsed_mins = ((now - last_check) / 60) as u32;

        let mut hunger_notification: Option<&'static str> = None;
        let mut should_save = false;

        // Only decrease during daytime, ~1 food per 10 minutes (lasts ~16 hours).
        if is_daytime && elapsed_mins >= 10 {
            let decrease = elapsed_mins / 10;
            let was_fed = pet.food > 0;
            pet.food = pet.food.saturating_sub(decrease);
            pet.last_food_check = Some(now);

            // Food just hit 0 - send notification and start death timer.
            if was_fed && pet.food == 0 {
                pet.hunger_zero_since = Some(now);
                pet.times_hungry = pet.times_hungry.saturating_add(1);
                hunger_notification = Some(pet.hunger_cry());
            }
            should_save = true;
        } else if !is_daytime {
            // Update timestamp during night so we don't accumulate.
            pet.last_food_check = Some(now);
        }

        // Check for death (3 hours = 10800 seconds at 0 food).
        let mut death_notification: Option<&'static str> = None;
        if pet.food == 0 {
            if let Some(zero_since) = pet.hunger_zero_since {
                if now - zero_since >= 10800 {
                    pet.is_dead = true;
                    death_notification = Some(pet.death_cry());
                    should_save = true;
                }
            }
        } else {
            pet.hunger_zero_since = None;
        }

        if let Some(msg) = hunger_notification {
            Self::send_notification("Tamadoro - HUNGRY!", msg);
        }
        if let Some(msg) = death_notification {
            Self::send_notification("Tamadoro - 💀", msg);
        }

        if should_save {
            self.save();
        }
    }
}
