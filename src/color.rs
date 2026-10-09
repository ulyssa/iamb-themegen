use std::sync::LazyLock;

use palette::color_difference::{Ciede2000, Wcag21RelativeContrast};
use palette::white_point::D65;
use palette::{IntoColor, Srgb};
use ratatui::prelude::*;

/// The CIELAB color space that we compare colors in.
pub type Lab = palette::Lab<D65, f64>;

/// Below this contrast ratio, text fails [WCAG 2.1 SC 1.4.3][sc143] at Level AA.
///
/// [sc143]: https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum
pub const CONTRAST_TEXT_MIN: f64 = 4.5;

/// Below this contrast ratio, even large text fails WCAG 2.1 SC 1.4.3, and we
/// consider the pair of colors unusable.
pub const CONTRAST_TEXT_FLOOR: f64 = 3.0;

/// How well a color reads against the background it is drawn on.
///
/// Assignment fills each slot from the best tier that has a candidate, and
/// `lint` reports the slots that had to settle, so the two agree on what
/// "legible" means.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Legibility {
    /// Too close in luminance to be made out.
    Poor,

    /// Readable as large text or an interface element (WCAG AA, 3:1).
    Fair,

    /// Readable as body text (WCAG AA, 4.5:1).
    Good,
}

/// The target for picking reds.
///
/// This avoids using `#ff0000` because that seems to result in bright orange
/// getting picked over light red/pink colors in practice.
pub static RED: LazyLock<Lab> = LazyLock::new(|| Srgb::new(0.9f64, 0.0, 0.0).into_color());

/// The target for picking greens.
///
/// We use a middle of the road green since too bright greens often end up being close to grey.
pub static GREEN: LazyLock<Lab> = LazyLock::new(|| Srgb::new(0.0f64, 0.5, 0.0).into_color());

/// The target for picking yellows.
pub static YELLOW: LazyLock<Lab> = LazyLock::new(|| Srgb::new(0.5f64, 0.5, 0.0).into_color());
pub static WHITE: LazyLock<Lab> = LazyLock::new(|| Srgb::new(1.0f64, 1.0, 1.0).into_color());
pub static BLACK: LazyLock<Lab> = LazyLock::new(|| Srgb::new(0.0f64, 0.0, 0.0).into_color());

/// A color from a colorscheme, together with its CIELAB representation.
#[derive(Clone, Copy)]
pub struct ThemeColor {
    pub rgb: (u8, u8, u8),
    pub lab: Lab,
}

impl ThemeColor {
    pub fn new(rgb: (u8, u8, u8)) -> Self {
        let lab = srgb(rgb).into_color();

        Self { rgb, lab }
    }

    pub fn color(&self) -> Color {
        let (r, g, b) = self.rgb;

        Color::Rgb(r, g, b)
    }

    /// The [WCAG 2.1 contrast ratio][wcag] between this color and another one.
    ///
    /// This ranges from 1:1 for two colors of identical luminance up to 21:1
    /// for black against white. Unlike [`get_delta_e_to_color`], this only
    /// considers luminance, which is what determines whether text rendered in
    /// one color is legible against the other.
    ///
    /// [wcag]: https://www.w3.org/TR/WCAG21/#dfn-contrast-ratio
    pub fn contrast(&self, other: &Self) -> f64 {
        srgb(self.rgb).relative_contrast(srgb(other.rgb))
    }

    /// Which tier of [`Legibility`] this color reaches against a background.
    pub fn legibility(&self, bg: &Self) -> Legibility {
        let contrast = self.contrast(bg);

        if contrast >= CONTRAST_TEXT_MIN {
            Legibility::Good
        } else if contrast >= CONTRAST_TEXT_FLOOR {
            Legibility::Fair
        } else {
            Legibility::Poor
        }
    }
}

/// Convert RGB components into the color space we do our math in.
fn srgb(rgb: (u8, u8, u8)) -> Srgb<f64> {
    let (r, g, b) = rgb;

    Srgb::new(r, g, b).into_format::<f64>()
}

/// Calculate the difference between two colors in the CIELAB color space.
pub fn get_delta_e_to_color(color: Lab, target: Lab) -> f64 {
    color.difference(target)
}

pub fn get_distance_to_red(color: Lab) -> f64 {
    get_delta_e_to_color(color, *RED)
}

pub fn get_distance_to_green(color: Lab) -> f64 {
    get_delta_e_to_color(color, *GREEN)
}

pub fn get_distance_to_yellow(color: Lab) -> f64 {
    get_delta_e_to_color(color, *YELLOW)
}

pub fn get_distance_to_white(color: Lab) -> f64 {
    get_delta_e_to_color(color, *WHITE)
}

pub fn get_distance_to_black(color: Lab) -> f64 {
    get_delta_e_to_color(color, *BLACK)
}
