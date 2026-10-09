use std::collections::BTreeSet;
use std::io::{BufWriter, Write};
use std::path::Path;

use ratatui::prelude::*;

use crate::ThemeFilter;
use crate::color::*;
use crate::de::*;

/// Colors that are closer together than this are considered too similar for us
/// to use them alongside each other since they wouldn't be visually distinguishable.
const MIN_DISTANCE: f64 = 4.0;

/// Sort colors by increasing distance from a target color.
fn sort_by_distance(colors: &mut [ThemeColor], f: impl Fn(Lab) -> f64) {
    colors.sort_by(|a, b| f(a.lab).total_cmp(&f(b.lab)).then(a.rgb.cmp(&b.rgb)));
}

/// Take the color that best matches a hue while still being readable.
///
/// Candidates that reach `target` against `bg` are preferred over ones that
/// don't, so a colorscheme with nothing legible to offer still gives up its
/// closest match rather than being thrown away. The RGB components break ties
/// so that our output is stable.
fn take_hue(
    colors: &mut Vec<ThemeColor>,
    bg: &ThemeColor,
    target: Legibility,
    distance: impl Fn(Lab) -> f64,
) -> Option<ThemeColor> {
    let key = |c: &ThemeColor| (c.legibility(bg) < target, distance(c.lab), c.rgb);

    let (i, _) = colors.iter().enumerate().min_by(|(_, a), (_, b)| {
        let (am, ad, ac) = key(a);
        let (bm, bd, bc) = key(b);

        am.cmp(&bm).then(ad.total_cmp(&bd)).then(ac.cmp(&bc))
    })?;

    Some(colors.remove(i))
}

/// Take the color that contrasts most with the backgrounds it gets drawn on.
fn take_contrasting(
    colors: &mut Vec<ThemeColor>,
    backgrounds: &[&ThemeColor],
) -> Option<ThemeColor> {
    let (i, _) = colors.iter().enumerate().min_by(|(_, a), (_, b)| {
        (-contrast(a, backgrounds))
            .total_cmp(&-contrast(b, backgrounds))
            .then(a.rgb.cmp(&b.rgb))
    })?;

    Some(colors.remove(i))
}

/// Take the least contrasting color that is `target` legibility.
///
/// Many colorschemes have fewer readable colors than we have slots to fill, so
/// a slot that grabs the strongest color it can find prevents us from being able
/// to use it for something where legibility may matter more. Taking only as much
/// contrast as the UI element needs leaves the rest available.
///
/// If nothing actually reaches the specified legibility then we just use the most
/// contrasting color.
fn take_legible(
    colors: &mut Vec<ThemeColor>,
    backgrounds: &[&ThemeColor],
    target: Legibility,
) -> Option<ThemeColor> {
    let meets = |c: &ThemeColor| backgrounds.iter().all(|bg| c.legibility(bg) >= target);
    let enough = colors.iter().any(meets);

    let key = |c: &ThemeColor| {
        let contrast = contrast(c, backgrounds);

        (!meets(c), if enough { contrast } else { -contrast }, c.rgb)
    };

    let (i, _) = colors.iter().enumerate().min_by(|(_, a), (_, b)| {
        let (am, ac, ar) = key(a);
        let (bm, bc, br) = key(b);

        am.cmp(&bm).then(ac.total_cmp(&bc)).then(ar.cmp(&br))
    })?;

    Some(colors.remove(i))
}

/// How well a color holds up against every background it gets drawn on.
fn contrast(color: &ThemeColor, backgrounds: &[&ThemeColor]) -> f64 {
    backgrounds
        .iter()
        .map(|bg| color.contrast(bg))
        .fold(f64::INFINITY, f64::min)
}

/// Take the pair of colors that the completion menu is drawn with.
///
/// The menu uses a reverse effect, so we actually wind up with `COLOR_BG3`
/// text on a `COLOR_UI1` background.
///
/// Since BG3 is only needed for the purposes of rendering the text on top
/// of UI1, it can come from the foreground set (unlike BG1 and BG2 which need
/// to use the colorscheme's actual set of background colors in order to
/// preserve the light/dark nature of the theme).
fn take_menu_pair(
    colors: &mut Vec<ThemeColor>,
    backgrounds: &[ThemeColor],
    bg1: &ThemeColor,
) -> Option<(ThemeColor, ThemeColor)> {
    let mut best: Option<(usize, ThemeColor)> = None;

    // Lower is better: a visible border first, since COLOR_UI1 has to work
    // against a background we have already committed to and COLOR_BG3 does
    // not, then a readable menu, then as much menu contrast as we can get, and
    // finally the components so that ties don't come down to iteration order.
    let rank = |ui1: &ThemeColor, bg3: &ThemeColor| {
        (
            ui1.legibility(bg1) < Legibility::Fair,
            bg3.legibility(ui1) < Legibility::Good,
            -bg3.contrast(ui1),
            ui1.rgb,
            bg3.rgb,
        )
    };

    for (i, ui1) in colors.iter().enumerate() {
        for bg3 in backgrounds {
            let better = best
                .as_ref()
                .map(|(j, chosen)| rank(ui1, bg3) < rank(&colors[*j], chosen))
                .unwrap_or(true);

            if better {
                best = Some((i, *bg3));
            }
        }
    }

    let (i, bg3) = best?;

    Some((colors.remove(i), bg3))
}

/// Given an input Vim colorscheme, assign its colors to the various COLOR_*
/// variables used with the theme template.
///
/// The COLOR_* variables are filled in order of how much their legibility matters,
/// because most colorschemes cannot offer a readable color for every one of them.
///
/// Note that being furthest away in CIELAB is not the same as being readable: a
/// saturated color can sit a long way from the background and still share
/// its luminance, so we use the WCAG ratio to assign by contrast instead.
fn colorscheme_assign(
    name: &str,
    colorscheme: &VimColorScheme,
    is_dark: bool,
    verbose: bool,
) -> Option<IambColorScheme> {
    let mut bgset = BTreeSet::new();
    let mut fgset = BTreeSet::new();

    for c in &colorscheme.colors {
        let Some(rgb) = rgb(c.hex_code) else {
            continue;
        };

        if c.name.ends_with("Bg") {
            bgset.insert(rgb);
        } else if c.name.ends_with("Fg") {
            fgset.insert(rgb);
        }
    }

    let lb = bgset.len();

    if lb < 3 {
        if verbose {
            eprintln!("Skipping {name} due to insufficient background colors ({lb})");
        }

        return None;
    }

    // Sort the backgrounds by increasing distance from all-white/all-black:
    let mut backgrounds: Vec<ThemeColor> = bgset.into_iter().map(ThemeColor::new).collect();

    if is_dark {
        sort_by_distance(&mut backgrounds, get_distance_to_black);
    } else {
        sort_by_distance(&mut backgrounds, get_distance_to_white);
    }

    let (bg1, bg2) = if is_dark {
        (backgrounds[0], backgrounds[1])
    } else {
        (backgrounds[1], backgrounds[0])
    };

    // Filter out colors too close to COLOR_BG1 from the foreground, as well as
    // the three backgrounds closest to the extreme. COLOR_BG3 is picked later,
    // alongside COLOR_UI1, but we keep excluding all three here so that the
    // size of the pool - and with it the decision to skip a scheme - doesn't
    // depend on how that pick turns out.
    let existing = [backgrounds[0].rgb, backgrounds[1].rgb, backgrounds[2].rgb];
    let mut others: Vec<ThemeColor> = fgset
        .into_iter()
        .map(ThemeColor::new)
        .filter(|c| {
            !existing.contains(&c.rgb) && get_delta_e_to_color(c.lab, bg1.lab) > MIN_DISTANCE
        })
        .collect();

    let lf = others.len();

    if lf < 9 {
        // Some color schemes just don't have that many colors, so we just
        // skip them here rather than generate something that would have
        // most usernames rendered in the same small set of colors.
        if verbose {
            eprintln!("Skipping {name} due to insufficient foreground colors ({lf})");
        }

        return None;
    }

    // Normal text is read constantly and is also drawn over COLOR_BG2 by the
    // selected completion item, so it goes first and takes the color that
    // stands out most against both backgrounds. Window titles are on screen
    // just as constantly, so they follow:
    let text1 = take_contrasting(&mut others, &[&bg1, &bg2])?;
    let ui2 = take_legible(&mut others, &[&bg1], Legibility::Good)?;

    // The informational colors come next. They have to match a hue as well as
    // be readable, which leaves them the least room, but they only need to be
    // legible rather than maximally so: they take the closest hue among the
    // colors that clear the bar for an icon.
    let green = take_hue(&mut others, &bg1, Legibility::Fair, get_distance_to_green)?;
    let red1 = take_hue(&mut others, &bg1, Legibility::Fair, get_distance_to_red)?;
    let yellow = take_hue(&mut others, &bg1, Legibility::Fair, get_distance_to_yellow)?;

    // COLOR_TEXT2 is drawn over COLOR_BG2 as well, by code blocks:
    let text2 = take_legible(&mut others, &[&bg1, &bg2], Legibility::Good)?;
    let text3 = take_legible(&mut others, &[&bg1], Legibility::Good)?;

    // At this point, we've used 9 foreground colors, and might be out!
    //
    // If we do have more, we'll try to use any remaining contrasting colors
    // for usernames, and some other text.
    let text4 = if lf > 9 {
        // We have plenty of colors, so we can use an extra one
        // for italicized text and room labels:
        take_legible(&mut others, &[&bg1], Legibility::Good)?
    } else {
        // Too few colors, just make `text4` the same as `text1`:
        text1
    };

    let text5 = if lf > 10 {
        // We have plenty of colors, so we can use an extra one
        // in room lists for read rooms:
        take_legible(&mut others, &[&bg1], Legibility::Good)?
    } else {
        // Too few colors, just make `text5` the same as `text1`:
        text1
    };

    // The second red only shows up on redacted messages, so it picks after the
    // text that is on screen all the time:
    let red2 = take_hue(&mut others, &bg1, Legibility::Fair, get_distance_to_red)?;

    // Finally, pick COLOR_UI1 and COLOR_BG3. COLOR_UI1 is primarily used for
    // window border colors, so it's okay if it's not fully legible against
    // BG1, but it is important that we pick a well-paired BG3 to ensure that
    // completions are readable.
    let (ui1, bg3) = take_menu_pair(&mut others, &backgrounds, &bg1)?;

    // Anything else we reuse for highlighting usernames, as long as they're
    // legible against the primary background:
    let mut color_plus: Vec<Color> = others
        .iter()
        .filter(|c| c.legibility(&bg1) >= Legibility::Fair)
        .map(ThemeColor::color)
        .collect();

    // Sort to ensure `.toml` output is stable:
    color_plus.sort_by_key(|c| rgb(*c));

    Some(IambColorScheme {
        color_bg1: bg1.color(),
        color_bg2: bg2.color(),
        color_bg3: bg3.color(),

        color_ui1: ui1.color(),
        color_ui2: ui2.color(),
        color_warn1: red1.color(),
        color_warn2: red2.color(),
        color_warn3: yellow.color(),
        color_good1: green.color(),
        color_text1: text1.color(),
        color_text2: text2.color(),
        color_text3: text3.color(),
        color_text4: text4.color(),
        color_text5: text5.color(),
        color_plus,
    })
}

fn assign_maybe(
    name: &str,
    colorscheme: Option<&VimColorScheme>,
    is_dark: bool,
    verbose: bool,
) -> Option<IambColorScheme> {
    colorscheme_assign(name, colorscheme?, is_dark, verbose)
}

fn assign_colorschemes(
    c: &ColorScheme<VimColorScheme>,
    verbose: bool,
) -> Option<ColorScheme<IambColorScheme>> {
    let name = c.name.as_str();
    let data = Backgrounds {
        light: assign_maybe(
            &format!("{name}-light"),
            c.data.light.as_ref(),
            false,
            verbose,
        ),
        dark: assign_maybe(&format!("{name}-dark"), c.data.dark.as_ref(), true, verbose),
    };

    if data.is_empty() {
        return None;
    }

    Some(ColorScheme {
        name: c.name.clone(),
        data,
    })
}

fn assign_repo(
    repo: &Repository<VimColorScheme>,
    verbose: bool,
) -> Option<Repository<IambColorScheme>> {
    let themes: Vec<_> = repo
        .themes
        .iter()
        .filter_map(|c| assign_colorschemes(c, verbose))
        .collect();

    if themes.is_empty() {
        return None;
    }

    Some(Repository {
        name: repo.name.clone(),
        owner: repo.owner.clone(),
        stargazers_count: repo.stargazers_count,
        themes,
    })
}

pub fn convert(path: &Path, filter: ThemeFilter, verbose: bool) -> anyhow::Result<()> {
    let repos = Repositories::<VimColorScheme>::load_file(path)?;
    let repos = filter.extract(repos);

    let mut output = Repositories::<IambColorScheme>::default();
    let converted = repos.repos.iter().flat_map(|r| assign_repo(r, verbose));
    output.repos.extend(converted);

    // Lock `Stdout` once instead of on every write `serde_json` makes:
    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    serde_json::to_writer_pretty(&mut out, &output)?;
    out.write_all(b"\n")?;
    out.flush()?;

    Ok(())
}
