use std::fs::File;
use std::path::Path;

use anyhow::Context;
use askama::Template;

use crate::de::{ColorScheme, IambColorScheme, Repositories, Repository, rgb};

#[derive(Clone, Template)]
#[template(path = "theme.txt")]
pub struct ThemeTemplate<'a> {
    name: &'a str,
    theme: &'a IambColorScheme,
}

fn generate_maybe(directory: &Path, name: &str, c: Option<&IambColorScheme>) -> anyhow::Result<()> {
    let Some(theme) = c else {
        return Ok(());
    };

    // Sort to ensure `.toml` output is stable:
    let mut theme = theme.clone();
    theme.color_plus.sort_by_key(|c| rgb(*c));

    let template = ThemeTemplate {
        name,
        theme: &theme,
    };

    std::fs::create_dir_all(directory)
        .with_context(|| format!("failed to create {}", directory.display()))?;

    let path = directory.join(format!("{name}.toml"));
    let mut file =
        File::create(&path).with_context(|| format!("failed to create {}", path.display()))?;

    template
        .write_into(&mut file)
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

fn generate_colorschemes(d: &Path, c: &ColorScheme<IambColorScheme>) -> anyhow::Result<()> {
    let name = c.name.as_str();

    generate_maybe(d, &format!("{name}-light"), c.data.light.as_ref())?;
    generate_maybe(d, &format!("{name}-dark"), c.data.dark.as_ref())?;

    Ok(())
}

/// Generate the theme files under a directory structure mirroring GitHub owner/repo.
///
/// This is done since the source JSON contains multiple repositories that contain themes
/// of the same name, which would cause us to overwrite output with different, slight
/// variations of the colorscheme.
fn generate_repo(output: &Path, repo: &Repository<IambColorScheme>) -> anyhow::Result<()> {
    let directory = output.join(&repo.owner.name).join(&repo.name);

    for colorschemes in &repo.themes {
        generate_colorschemes(&directory, colorschemes)?;
    }

    Ok(())
}

pub fn generate(path: &Path, output: &Path) -> anyhow::Result<()> {
    let repos = Repositories::<IambColorScheme>::load_file(path)?;

    for repo in &repos.repos {
        generate_repo(output, repo)?;
    }

    Ok(())
}
