use std::time::{Duration, Instant};

use rand::{seq::SliceRandom, Rng};

use crate::game::{GameData, PetMood, PetType};

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Timer,
    Pet,
    Stats,
    Debug,
}

#[derive(PartialEq, Clone, Copy)]
pub enum PomodoroState {
    Work,
    Break,
    Paused,
}

pub struct App {
    pub mode: Mode,
    pub pomo_state: PomodoroState,
    pub pomo_remaining: Duration,
    pub pomo_total: Duration,
    pub pomo_sessions: u32,
    pub last_tick: Instant,
    pub game: GameData,
    pub frame: usize,
    pub message: Option<(String, Instant)>,
    pub paused_from_state: Option<PomodoroState>, // Track what state we paused from
    pub test_mode: bool,                           // Whether --test flag was passed
    pub pet_speech: Option<(String, Instant)>,     // Current speech and when it started
    pub next_speech_time: Instant,                 // When the next speech should trigger
}

impl App {
    pub fn new(test_mode: bool) -> Self {
        let work_duration = Duration::from_secs(25 * 60);
        let now = Instant::now();
        // First speech in 30-60 seconds
        let next_speech_delay = Duration::from_secs(rand::thread_rng().gen_range(30..60));

        App {
            mode: Mode::Timer,
            pomo_state: PomodoroState::Paused,
            pomo_remaining: work_duration,
            pomo_total: work_duration,
            pomo_sessions: 0,
            last_tick: now,
            game: GameData::load(),
            frame: 0,
            message: None,
            paused_from_state: None,
            test_mode,
            pet_speech: None,
            next_speech_time: now + next_speech_delay,
        }
    }

    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);

        // Update food (decreases during daytime)
        self.game.update_food();

        // Clear old messages
        if let Some((_, time)) = &self.message {
            if time.elapsed() > Duration::from_secs(3) {
                self.message = None;
            }
        }

        // Handle pet speech
        if let Some((_, time)) = &self.pet_speech {
            // Clear speech after 5 seconds
            if time.elapsed() > Duration::from_secs(5) {
                self.pet_speech = None;
            }
        } else {
            // Check if it's time for new speech
            if Instant::now() >= self.next_speech_time {
                let phrase = self.get_pet_phrase();
                self.pet_speech = Some((phrase.to_string(), Instant::now()));
                // Next speech in 45-120 seconds
                self.next_speech_time = Instant::now()
                    + Duration::from_secs(rand::thread_rng().gen_range(45..120));
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
                    let old_level = self.game.pet().level;
                    let old_stage = crate::game::Pet::evolution_stage_for_level(old_level);
                    self.game.record_session();

                    let new_level = self.game.pet().level;
                    let new_stage = self.game.pet().evolution_stage();
                    if new_level > old_level {
                        self.message = Some((
                            format!("LEVEL UP! Lv.{}", new_level),
                            Instant::now(),
                        ));
                        if new_stage > old_stage {
                            self.message = Some((
                                format!("{} evolved!", self.game.pet().name),
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
                        let msg =
                            messages[rand::thread_rng().gen_range(0..messages.len())];
                        self.message = Some((msg.to_string(), Instant::now()));
                    }

                    self.pomo_state = PomodoroState::Break;
                    self.pomo_total = Duration::from_secs(5 * 60);
                    self.pomo_remaining = self.pomo_total;
                    self.paused_from_state = None; // Clear saved state on natural transition
                    self.game.pet_mut().mood = PetMood::Resting;
                } else {
                    self.pomo_state = PomodoroState::Paused;
                    self.pomo_total = Duration::from_secs(25 * 60);
                    self.pomo_remaining = self.pomo_total;
                    self.paused_from_state = None; // Clear saved state on natural transition
                    self.game.pet_mut().mood = PetMood::Idle;
                    self.message = Some(("Break over! Ready?".to_string(), Instant::now()));
                }
            }
        }
    }

    pub fn toggle_pomo(&mut self) {
        match self.pomo_state {
            PomodoroState::Paused => {
                // Resume to the state we paused from, or default to Work
                let resume_state = self.paused_from_state.unwrap_or(PomodoroState::Work);
                self.pomo_state = resume_state;
                self.paused_from_state = None; // Clear the saved state
                self.last_tick = Instant::now();

                // Set mood based on what state we're resuming to
                self.game.pet_mut().mood = match resume_state {
                    PomodoroState::Work => PetMood::Working,
                    PomodoroState::Break => PetMood::Resting,
                    PomodoroState::Paused => PetMood::Idle,
                };
                self.game.save();
            }
            _ => {
                // Save the current state before pausing
                self.paused_from_state = Some(self.pomo_state);
                self.pomo_state = PomodoroState::Paused;
                self.game.pet_mut().mood = PetMood::Idle;
                self.game.save();
            }
        }
    }

    pub fn reset_pomo(&mut self) {
        self.pomo_state = PomodoroState::Paused;
        self.pomo_total = Duration::from_secs(25 * 60);
        self.pomo_remaining = self.pomo_total;
        self.paused_from_state = None; // Clear saved state on reset
        self.game.pet_mut().mood = PetMood::Idle;
        self.game.save();
    }

    pub fn get_pet_phrase(&self) -> &'static str {
        use PetMood::*;
        use PetType::*;

        let pet = self.game.pet();

        // Special cases first
        if pet.is_dead {
            return match pet.pet_type {
                Blob => "...",
                Cat => "*silence*",
                Robot => "ERROR 404",
                Ghost => "......",
            };
        }

        if pet.food < 20 {
            return match pet.pet_type {
                Blob => "I'm hungry...",
                Cat => "Meow! Feed me!",
                Robot => "Low energy!",
                Ghost => "Need food...",
            };
        }

        match pet.mood {
            Working => match pet.pet_type {
                Blob => *["You got this!", "Keep going!", "Focus!", "Stay strong!"]
                    .choose(&mut rand::thread_rng())
                    .unwrap(),
                Cat => *[
                    "Purr... working hard?",
                    "Meow! Stay focused!",
                    "You're doing great!",
                    "Keep it up, human!",
                ]
                .choose(&mut rand::thread_rng())
                .unwrap(),
                Robot => *[
                    "PROCESSING...",
                    "Productivity mode!",
                    "Executing tasks!",
                    "System optimal!",
                ]
                .choose(&mut rand::thread_rng())
                .unwrap(),
                Ghost => *[
                    "Boo! Keep working!",
                    "You can do it!",
                    "Floating by to cheer!",
                    "Focus time!",
                ]
                .choose(&mut rand::thread_rng())
                .unwrap(),
            },
            Happy => match pet.pet_type {
                Blob => *["Yay! We did it!", "So happy!", "Woohoo!", "Great work!"]
                    .choose(&mut rand::thread_rng())
                    .unwrap(),
                Cat => *["Purrrr~", "Meow! ^w^", "Head pats please!", "You're the best!"]
                    .choose(&mut rand::thread_rng())
                    .unwrap(),
                Robot => *[
                    "SUCCESS!",
                    "Task completed!",
                    "Happiness.exe",
                    "Achievement unlocked!",
                ]
                .choose(&mut rand::thread_rng())
                .unwrap(),
                Ghost => *["Boo! Yay!", "Spooky happy!", "We did it!", "Ghostly cheer!"]
                    .choose(&mut rand::thread_rng())
                    .unwrap(),
            },
            Resting => match pet.pet_type {
                Blob => *["zzz...", "Nice break...", "Relaxing~", "So comfy..."]
                    .choose(&mut rand::thread_rng())
                    .unwrap(),
                Cat => *["*yawn* meow", "Nap time!", "zzz purr zzz", "Sleep well~"]
                    .choose(&mut rand::thread_rng())
                    .unwrap(),
                Robot => *[
                    "Sleep mode...",
                    "Recharging...",
                    "Systems idle...",
                    "zzz beep zzz",
                ]
                .choose(&mut rand::thread_rng())
                .unwrap(),
                Ghost => *[
                    "Floating dreams...",
                    "zzz boo zzz",
                    "Resting~",
                    "Peaceful...",
                ]
                .choose(&mut rand::thread_rng())
                .unwrap(),
            },
            Idle => match pet.pet_type {
                Blob => *["Hello!", "Ready when you are!", "Let's work!", "How are you?"]
                    .choose(&mut rand::thread_rng())
                    .unwrap(),
                Cat => *["Meow!", "Pet me!", "*stares*", "Let's go!"]
                    .choose(&mut rand::thread_rng())
                    .unwrap(),
                Robot => *["READY.", "Standing by!", "Awaiting input!", "Hello, human!"]
                    .choose(&mut rand::thread_rng())
                    .unwrap(),
                Ghost => *["Boo!", "Floating here~", "Ready to help!", "Let's start!"]
                    .choose(&mut rand::thread_rng())
                    .unwrap(),
            },
        }
    }
}
