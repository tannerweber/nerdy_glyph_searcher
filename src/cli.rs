use clap::{CommandFactory, Parser, ValueEnum};
use std::io;

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

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Shell {
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
    pub fn print_completion(self) {
        fn g(shell: clap_complete::shells::Shell) {
            clap_complete::generate(
                shell,
                &mut Args::command(),
                EXE_NAME,
                &mut io::stdout(),
            );
        }

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
                    &mut io::stdout(),
                );
            }
        }
    }
}
