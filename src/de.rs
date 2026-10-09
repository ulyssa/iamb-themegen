use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::Context;
use ratatui::prelude::*;
use serde::de::DeserializeOwned;

pub fn rgb(color: Color) -> Option<(u8, u8, u8)> {
    if let Color::Rgb(r, g, b) = color {
        Some((r, g, b))
    } else {
        None
    }
}

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VimColor {
    pub name: String,
    pub hex_code: Color,
}

#[derive(Clone, serde::Deserialize)]
#[serde(transparent)]
pub struct VimColorScheme {
    pub colors: Vec<VimColor>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct IambColorScheme {
    pub color_bg1: Color,
    pub color_bg2: Color,
    pub color_bg3: Color,

    pub color_ui1: Color,
    pub color_ui2: Color,
    pub color_warn1: Color,
    pub color_warn2: Color,
    pub color_warn3: Color,
    pub color_good1: Color,
    pub color_text1: Color,
    pub color_text2: Color,
    pub color_text3: Color,
    pub color_text4: Color,
    pub color_text5: Color,
    #[serde(default)]
    pub color_plus: Vec<Color>,
}

/// The light and dark variants of a colorscheme.
#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(bound(deserialize = "C: serde::Deserialize<'de>"))]
pub struct Backgrounds<C> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light: Option<C>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dark: Option<C>,
}

impl<C> Backgrounds<C> {
    pub fn is_empty(&self) -> bool {
        self.light.is_none() && self.dark.is_none()
    }
}

impl<C> Default for Backgrounds<C> {
    fn default() -> Self {
        Self {
            light: None,
            dark: None,
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct ColorScheme<C> {
    pub name: String,
    pub data: Backgrounds<C>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Owner {
    pub name: String,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Repository<C> {
    pub name: String,
    pub owner: Owner,

    #[serde(rename = "stargazersCount", default)]
    pub stargazers_count: usize,

    #[serde(rename = "vimColorSchemes")]
    pub themes: Vec<ColorScheme<C>>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Repositories<C> {
    pub repos: Vec<Repository<C>>,
}

impl<C> Repositories<C>
where
    C: DeserializeOwned,
{
    pub fn load_file(path: &Path) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).context("failed to parse input JSON")
    }
}

impl<C> Default for Repositories<C> {
    fn default() -> Self {
        Self { repos: vec![] }
    }
}
