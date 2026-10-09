use std::path::PathBuf;

mod color;
mod convert;
mod de;
mod generate;

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Convert a summary of Vim colorschemes to an iamb themes collection.
    Convert {
        /// Print the Vim colorschemes that get skipped to stderr
        #[clap(long, short)]
        verbose: bool,

        #[clap(flatten)]
        filter: ThemeFilter,

        /// Input JSON file
        file: PathBuf,
    },

    /// Generate the iamb `theme.toml` files from a JSON collection of themes.
    Generate {
        /// Directory to write the generated themes into
        #[clap(long, short, default_value = "themes")]
        output: PathBuf,

        /// Input JSON file
        file: PathBuf,
    },
}

#[derive(Clone, Eq, PartialEq, clap::Args)]
pub struct ThemeFilter {
    /// Filter for themes with a matching background palette.
    #[clap(long)]
    bg: Option<Palette>,

    /// Filter output by number of stars on the repository.
    #[clap(long, default_value_t = 0)]
    min_stars: usize,

    /// Filter output by repository owner substring.
    #[clap(long)]
    owner: Option<String>,

    /// Filter output by repository name substring.
    #[clap(long)]
    repo: Option<String>,

    /// Filter output by a substring within the colorscheme's name.
    #[clap(long)]
    name: Option<String>,
}

impl ThemeFilter {
    fn extract<C>(&self, repositories: de::Repositories<C>) -> de::Repositories<C> {
        let mut repos = vec![];

        for mut r in repositories.repos {
            if r.stargazers_count < self.min_stars {
                continue;
            }

            if let Some(f) = self.owner.as_deref()
                && !r.owner.name.contains(f)
            {
                continue;
            }

            if let Some(f) = self.repo.as_deref()
                && !r.name.contains(f)
            {
                continue;
            }

            let mut colorschemes = vec![];

            for mut c in std::mem::take(&mut r.themes) {
                if let Some(f) = self.name.as_deref()
                    && !c.name.contains(f)
                {
                    continue;
                }

                match self.bg {
                    None => {
                        colorschemes.push(c);
                    }
                    Some(Palette::Light) => {
                        if c.data.light.is_none() {
                            continue;
                        }

                        c.data.dark = None;
                        colorschemes.push(c);
                    }
                    Some(Palette::Dark) => {
                        if c.data.dark.is_none() {
                            continue;
                        }

                        c.data.light = None;
                        colorschemes.push(c);
                    }
                }
            }

            if colorschemes.is_empty() {
                continue;
            }

            r.themes = colorschemes;
            repos.push(r);
        }

        de::Repositories { repos }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, clap::ValueEnum)]
pub enum Palette {
    Light,
    Dark,
}

#[derive(clap::Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Subcommand,
}

fn main() -> anyhow::Result<()> {
    let cli = <Cli as clap::Parser>::parse();

    match cli.command {
        Subcommand::Convert {
            file,
            filter,
            verbose,
        } => convert::convert(&file, filter, verbose),
        Subcommand::Generate { file, output } => generate::generate(&file, &output),
    }
}
