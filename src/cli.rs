use clap::{CommandFactory, Parser, ValueEnum};
use std::{io::Write, vec};

const EXE_NAME: &str = "nerdy_glyph_searcher";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Search your terminal font for glyphs by name from the command line
pub struct Args {
    /// An explicit ttf file to search
    #[arg(short, long)]
    pub font: Option<String>,

    /// Use verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Send results to stdout instead of interactive mode
    #[arg(short, long)]
    pub output: bool,

    /// Generate and print shell completion for the specified shell
    #[arg(long, name = "shell-completion", value_name = "SHELL")]
    pub shell_completion: Option<Shell>,
}

#[derive(Debug, Clone, ValueEnum, Default, Eq, PartialEq)]
pub enum Shell {
    #[default]
    Bash,
    Elvish,
    Fish,
    #[allow(clippy::enum_variant_names)]
    #[clap(name = "powershell", alias = "pwsh", alias = "power-shell")]
    PowerShell,
    Zsh,
    Nushell,
}

impl Shell {
    pub fn write_completion(&self, w: &mut dyn Write) {
        let mut g = |shell: clap_complete::shells::Shell| {
            clap_complete::generate(shell, &mut Args::command(), EXE_NAME, w);
        };

        match self {
            Self::Bash => {
                g(clap_complete::shells::Shell::Bash);
            }
            Self::Elvish => {
                g(clap_complete::shells::Shell::Elvish);
            }
            Self::Fish => {
                g(clap_complete::shells::Shell::Fish);
            }
            Self::PowerShell => {
                g(clap_complete::shells::Shell::PowerShell);
            }
            Self::Zsh => {
                g(clap_complete::shells::Shell::Zsh);
            }
            Self::Nushell => {
                clap_complete::generate(
                    clap_complete_nushell::Nushell,
                    &mut Args::command(),
                    EXE_NAME,
                    w,
                );
            }
        }
    }

    #[allow(unused)]
    #[must_use]
    fn get_all() -> Vec<Self> {
        vec![
            Self::Bash,
            Self::Elvish,
            Self::Fish,
            Self::PowerShell,
            Self::Zsh,
            Self::Nushell,
        ]
    }

    #[allow(unused)]
    #[must_use]
    fn extension(self) -> String {
        match self {
            Self::Bash => ".bash",
            Self::Elvish => ".elv",
            Self::Fish => ".fish",
            Self::PowerShell => ".ps1",
            Self::Zsh => ".zsh",
            Self::Nushell => ".nu",
        }
        .to_string()
    }
}
