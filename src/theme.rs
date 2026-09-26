use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

pub const BACKGROUND: Color = Color::Rgb(13, 16, 20);
pub const PANEL: Color = Color::Rgb(20, 24, 29);
pub const PANEL_ALT: Color = Color::Rgb(27, 32, 38);
pub const FOREGROUND: Color = Color::Rgb(226, 232, 235);
pub const MUTED: Color = Color::Rgb(133, 145, 153);
pub const BORDER: Color = Color::Rgb(55, 66, 74);
pub const ACCENT: Color = Color::Rgb(76, 204, 183);
pub const SELECTED_BACKGROUND: Color = Color::Rgb(31, 52, 54);
pub const URGENT: Color = Color::Rgb(255, 92, 104);
pub const HIGH: Color = Color::Rgb(247, 163, 77);
pub const MEDIUM: Color = Color::Rgb(106, 173, 245);
pub const LOW: Color = Color::Rgb(114, 203, 133);
pub const ON_TRACK: Color = Color::Rgb(114, 203, 133);
pub const TIGHT: Color = Color::Rgb(244, 205, 91);
pub const UNKNOWN: Color = Color::Rgb(133, 145, 153);
pub const DONE: Color = Color::Rgb(93, 105, 112);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub theme: ThemeConfig,
    pub keybindings: KeyBindings,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub background: String,
    pub panel: String,
    pub panel_alt: String,
    pub foreground: String,
    pub muted: String,
    pub border: String,
    pub accent: String,
    pub selected_background: String,
    pub urgent: String,
    pub high: String,
    pub medium: String,
    pub low: String,
    pub on_track: String,
    pub tight: String,
    pub unknown: String,
    pub done: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct KeyBindings {
    pub down: String,
    pub up: String,
    pub filter: String,
    pub sort: String,
    pub search: String,
    pub add: String,
    pub delete: String,
    pub cycle_state: String,
    pub help: String,
    pub quit: String,
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: Color,
    pub panel: Color,
    pub panel_alt: Color,
    pub foreground: Color,
    pub muted: Color,
    pub border: Color,
    pub accent: Color,
    pub selected_background: Color,
    pub urgent: Color,
    pub high: Color,
    pub medium: Color,
    pub low: Color,
    pub on_track: Color,
    pub tight: Color,
    pub unknown: Color,
    pub done: Color,
}

impl Config {
    pub fn load_or_create() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("reading config {}", path.display()))?;
            return toml::from_str(&text)
                .with_context(|| format!("parsing config {}", path.display()));
        }

        let config = Self::default();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, toml::to_string_pretty(&config)?)
            .with_context(|| format!("writing config {}", path.display()))?;
        Ok(config)
    }

    fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("could not locate the home directory")?;
        Ok(home.join(".config").join("taskdeck").join("config.toml"))
    }

    pub fn default_db_path() -> Result<PathBuf> {
        crate::storage::default_database_path()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            keybindings: KeyBindings::default(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background: "#0d1014".to_owned(),
            panel: "#14181d".to_owned(),
            panel_alt: "#1b2026".to_owned(),
            foreground: "#e2e8eb".to_owned(),
            muted: "#859199".to_owned(),
            border: "#37424a".to_owned(),
            accent: "#4cccb7".to_owned(),
            selected_background: "#1f3436".to_owned(),
            urgent: "#ff5c68".to_owned(),
            high: "#f7a34d".to_owned(),
            medium: "#6aadf5".to_owned(),
            low: "#72cb85".to_owned(),
            on_track: "#72cb85".to_owned(),
            tight: "#f4cd5b".to_owned(),
            unknown: "#859199".to_owned(),
            done: "#5d6970".to_owned(),
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            down: "j".to_owned(),
            up: "k".to_owned(),
            filter: "f".to_owned(),
            sort: "o".to_owned(),
            search: "/".to_owned(),
            add: "a".to_owned(),
            delete: "d".to_owned(),
            cycle_state: " ".to_owned(),
            help: "?".to_owned(),
            quit: "q".to_owned(),
        }
    }
}

impl Theme {
    pub fn from_config(config: &ThemeConfig) -> Self {
        Self {
            background: color(&config.background, BACKGROUND),
            panel: color(&config.panel, PANEL),
            panel_alt: color(&config.panel_alt, PANEL_ALT),
            foreground: color(&config.foreground, FOREGROUND),
            muted: color(&config.muted, MUTED),
            border: color(&config.border, BORDER),
            accent: color(&config.accent, ACCENT),
            selected_background: color(&config.selected_background, SELECTED_BACKGROUND),
            urgent: color(&config.urgent, URGENT),
            high: color(&config.high, HIGH),
            medium: color(&config.medium, MEDIUM),
            low: color(&config.low, LOW),
            on_track: color(&config.on_track, ON_TRACK),
            tight: color(&config.tight, TIGHT),
            unknown: color(&config.unknown, UNKNOWN),
            done: color(&config.done, DONE),
        }
    }
}

fn color(value: &str, fallback: Color) -> Color {
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 {
        return fallback;
    }
    let Ok(rgb) = u32::from_str_radix(hex, 16) else {
        return fallback;
    };
    Color::Rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}