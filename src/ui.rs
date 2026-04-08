use chrono::Local;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph},
};

use crate::app::{App, Mode, PomodoroState};
use crate::ascii_digits;
use crate::colors;
use crate::game::PetMood;

pub fn render_large_clock(f: &mut Frame, area: Rect) {
    let now = Local::now();
    let time_str = format!("{}", now.format("%H%M")); // Remove colon

    // Build the 5 lines of the clock
    let mut lines: Vec<String> = vec![String::new(); 5];

    for (idx, ch) in time_str.chars().enumerate() {
        let digit_art = ascii_digits::get_digit(ch);
        for (i, line) in digit_art.iter().enumerate() {
            lines[i].push_str(line);
            // Add spacing: 1 space within groups, 2 spaces between hour and minute
            if idx == 0 {
                // After first hour digit
                lines[i].push(' ');
            } else if idx == 1 {
                // After second hour digit (between hours and minutes) - 2 spaces for separation
                lines[i].push_str("  ");
            } else if idx == 2 {
                // After first minute digit
                lines[i].push(' ');
            }
            // idx == 3 (last digit) gets no space
        }
    }

    // Create a layout to vertically center the clock
    // Calculate explicit padding (area is 7 lines, clock is 5 lines, so 2 lines to split)
    let clock_height = 5;
    let total_padding = area.height.saturating_sub(clock_height);
    let top_padding = total_padding / 2;
    let bottom_padding = total_padding - top_padding;

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_padding),    // Top padding (explicit)
            Constraint::Length(clock_height),   // Clock (5 lines)
            Constraint::Length(bottom_padding), // Bottom padding (explicit)
        ])
        .split(area);

    let clock_text: Vec<Line> = lines
        .iter()
        .map(|line| Line::from(Span::styled(line, Style::default().fg(colors::FG))))
        .collect();

    f.render_widget(
        Paragraph::new(clock_text).alignment(Alignment::Center),
        v_chunks[1],
    );
}

pub fn ui(f: &mut Frame, app: &App) {
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
            Constraint::Min(10),   // Content (minimum height)
            Constraint::Min(5),    // Clock area (takes remaining space, min 5 for clock)
            Constraint::Length(1), // Message/Help
        ])
        .split(inner);

    // Tabs
    let mut tab_spans = vec![
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
    ];

    // Only show DEBUG tab when in test mode
    if app.test_mode {
        tab_spans.push(Span::raw(" "));
        tab_spans.push(Span::styled(
            "DEBUG",
            if app.mode == Mode::Debug {
                Style::default().fg(colors::BG).bg(colors::RED).bold()
            } else {
                Style::default().fg(colors::COMMENT)
            },
        ));
    }

    let tabs = Line::from(tab_spans);
    f.render_widget(Paragraph::new(tabs).alignment(Alignment::Center), chunks[0]);

    match app.mode {
        Mode::Timer => render_timer(f, chunks[2], app),
        Mode::Pet => render_pet(f, chunks[2], app),
        Mode::Stats => render_stats(f, chunks[2], app),
        Mode::Debug => render_debug(f, chunks[2], app),
    }

    // Large ASCII Clock
    render_large_clock(f, chunks[3]);

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
        chunks[4],
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

    // Mini pet with optional speech
    let pet_color = if app.pomo_state == PomodoroState::Work {
        colors::RED
    } else if app.pomo_state == PomodoroState::Break {
        colors::GREEN
    } else {
        colors::CYAN
    };

    if let Some((speech, _)) = &app.pet_speech {
        render_pet_with_speech(f, chunks[0], app, speech, pet_color);
    } else {
        let pet_art = app.game.pet().get_art(app.frame / 2);
        let pet_text: Vec<Line> = pet_art
            .iter()
            .map(|line| Line::from(Span::styled(*line, Style::default().fg(pet_color))))
            .collect();
        f.render_widget(
            Paragraph::new(pet_text).alignment(Alignment::Center),
            chunks[0],
        );
    }

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
    let pet = app.game.pet();
    let xp_progress = pet.xp as f64 / pet.xp_to_next_level() as f64;
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(colors::YELLOW))
            .ratio(xp_progress)
            .label(format!("Lv.{}", pet.level)),
        chunks[4],
    );
}

fn render_pet_with_speech(f: &mut Frame, area: Rect, app: &App, speech: &str, pet_color: Color) {
    // Create horizontal layout: left padding, pet, speech bubble, right padding
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Left padding
            Constraint::Length(10),     // Pet
            Constraint::Length(20),     // Speech bubble
            Constraint::Min(0),         // Right padding
        ])
        .split(area);

    // Render pet
    let pet_art = app.game.pet().get_art(app.frame / 2);
    let pet_text: Vec<Line> = pet_art
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(pet_color))))
        .collect();
    f.render_widget(
        Paragraph::new(pet_text).alignment(Alignment::Center),
        h_chunks[1],
    );

    // Render speech bubble
    let bubble_border = "─".repeat(speech.len().min(18));
    let speech_trimmed = if speech.len() > 18 {
        format!("{}...", &speech[..15])
    } else {
        speech.to_string()
    };

    let bubble_text = vec![
        Line::from(format!("┌{}┐", bubble_border)),
        Line::from(format!("│{}│", speech_trimmed)),
        Line::from(format!("└{}┘", bubble_border)),
    ];

    f.render_widget(
        Paragraph::new(bubble_text)
            .style(Style::default().fg(colors::FG))
            .alignment(Alignment::Left),
        h_chunks[2],
    );
}

fn render_pet(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Name
            Constraint::Length(5), // Pet art + speech bubble
            Constraint::Length(1), // Stage
            Constraint::Length(1), // Level
            Constraint::Length(1), // XP
            Constraint::Length(1), // XP bar
            Constraint::Length(1), // Food
            Constraint::Length(1), // Food bar
            Constraint::Min(0),    // Spacer
        ])
        .split(area);

    let pet = app.game.pet();
    let mood_color = match pet.mood {
        PetMood::Working => colors::RED,
        PetMood::Happy => colors::YELLOW,
        PetMood::Resting => colors::GREEN,
        PetMood::Idle => colors::CYAN,
    };

    // Name
    f.render_widget(
        Paragraph::new(format!("~ {} ~", pet.name))
            .style(Style::default().fg(colors::CYAN).bold())
            .alignment(Alignment::Center),
        chunks[0],
    );

    // Pet art with speech bubble
    if let Some((speech, _)) = &app.pet_speech {
        render_pet_with_speech(f, chunks[1], app, speech, mood_color);
    } else {
        let pet_art = pet.get_art(app.frame / 2);
        let pet_text: Vec<Line> = pet_art
            .iter()
            .map(|line| Line::from(Span::styled(*line, Style::default().fg(mood_color))))
            .collect();
        f.render_widget(
            Paragraph::new(pet_text).alignment(Alignment::Center),
            chunks[1],
        );
    }

    // Stage + Type
    f.render_widget(
        Paragraph::new(format!("{} {}", pet.pet_type.name(), pet.stage_name()))
            .style(Style::default().fg(colors::MAGENTA))
            .alignment(Alignment::Center),
        chunks[2],
    );

    // Level
    f.render_widget(
        Paragraph::new(format!("Level {}", pet.level))
            .style(Style::default().fg(colors::FG).bold())
            .alignment(Alignment::Center),
        chunks[3],
    );

    // XP
    f.render_widget(
        Paragraph::new(format!("XP: {}/{}", pet.xp, pet.xp_to_next_level()))
            .style(Style::default().fg(colors::YELLOW))
            .alignment(Alignment::Center),
        chunks[4],
    );

    // XP progress bar
    let xp_progress = pet.xp as f64 / pet.xp_to_next_level() as f64;
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(colors::YELLOW))
            .ratio(xp_progress)
            .label(""),
        chunks[5],
    );

    // Food
    f.render_widget(
        Paragraph::new(format!("Food: {}/100", pet.food))
            .style(Style::default().fg(colors::GREEN))
            .alignment(Alignment::Center),
        chunks[6],
    );

    // Food bar
    let food_progress = pet.food as f64 / 100.0;
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(if pet.food > 30 {
                colors::GREEN
            } else if pet.food > 10 {
                colors::YELLOW
            } else {
                colors::RED
            }))
            .ratio(food_progress)
            .label(""),
        chunks[7],
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
            Constraint::Length(1), // Level/XP
            Constraint::Length(1), // Stage
            Constraint::Length(1), // Food
            Constraint::Length(1), // Streak
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Controls header
            Constraint::Length(1), // Control 1
            Constraint::Length(1), // Control 2
            Constraint::Length(1), // Control 3
            Constraint::Length(1), // Control 4
            Constraint::Length(1), // Control 5
            Constraint::Length(1), // Control 6
            Constraint::Length(1), // Control 7
            Constraint::Min(0),    // Spacer
        ])
        .split(area);

    f.render_widget(
        Paragraph::new("Debug Panel")
            .style(Style::default().fg(colors::RED).bold())
            .alignment(Alignment::Center),
        chunks[0],
    );

    let pet = app.game.pet();
    f.render_widget(
        Paragraph::new(format!(
            "Lv.{} | XP: {}/{}",
            pet.level,
            pet.xp,
            pet.xp_to_next_level()
        ))
        .style(Style::default().fg(colors::FG))
        .alignment(Alignment::Center),
        chunks[2],
    );

    f.render_widget(
        Paragraph::new(format!(
            "{} {} (Stg {})",
            pet.pet_type.name(),
            pet.stage_name(),
            pet.evolution_stage()
        ))
        .style(Style::default().fg(colors::CYAN))
        .alignment(Alignment::Center),
        chunks[3],
    );

    let food_text = if pet.is_dead {
        "DEAD 💀".to_string()
    } else {
        format!("Food: {}/100", pet.food)
    };
    f.render_widget(
        Paragraph::new(food_text)
            .style(Style::default().fg(if pet.is_dead {
                colors::RED
            } else if pet.food > 30 {
                colors::GREEN
            } else if pet.food > 10 {
                colors::YELLOW
            } else {
                colors::RED
            }))
            .alignment(Alignment::Center),
        chunks[4],
    );

    f.render_widget(
        Paragraph::new(format!(
            "Streak: {} | Sessions: {}",
            app.game.streak_days, app.game.total_sessions
        ))
        .style(Style::default().fg(colors::YELLOW))
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
        Paragraph::new("7: +Food 8: -Food")
            .style(Style::default().fg(colors::GREEN))
            .alignment(Alignment::Center),
        chunks[11],
    );

    f.render_widget(
        Paragraph::new("9: Kill/Revive")
            .style(
                Style::default().fg(if app.game.pet().is_dead {
                    colors::GREEN
                } else {
                    colors::RED
                }),
            )
            .alignment(Alignment::Center),
        chunks[12],
    );

    f.render_widget(
        Paragraph::new("0: RESET ALL")
            .style(Style::default().fg(colors::RED))
            .alignment(Alignment::Center),
        chunks[13],
    );
}
