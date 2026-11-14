// SPDX-License-Identifier: AGPL-3.0-or-later
//
// This file is part of Nerdy Glyph Searcher.
//
// Nerdy Glyph Searcher is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Nerdy Glyph Searcher is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Foobar. If not, see <https://www.gnu.org/licenses/>.

mod cli;

use clap::Parser;
use rayon::prelude::*;
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    string::ToString,
    sync::mpsc::channel,
};
use ttf_parser::{Face, GlyphId, cmap::Subtable};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let args = cli::Args::parse();

    if let Some(shell) = args.shell_completion {
        shell.write_completion(&mut std::io::stdout());
        return Ok(());
    }

    let font_path = if let Some(path) = args.font {
        path.into()
    } else {
        let terminal_program = Terminal::identify_terminal()?;
        let font_path = terminal_program.get_terminal_font_file()?;
        if args.verbose {
            eprintln!("Terminal: {terminal_program}");
        }
        font_path
    };

    if args.verbose {
        eprintln!("Font Path: {}", font_path.display());
    }

    if args.output {
        println!("{}", get_glyph_name_list(&font_path)?);
    } else {
        let mut child = Command::new(Dependency::Fzf.exe()?)
            .args([
                "--exact",
                "--layout=reverse",
                "--border=rounded",
                "--padding=1",
                "--border-label= Nerdy Glyph Searcher ",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let mut stdin = child.stdin.take().ok_or(Error::FzfSpawn)?;
        let (tx, rx) = channel();
        let names = get_glyph_name_list(&font_path)?;

        if tx.send(names).is_err() {
            return Err(Error::Send);
        }

        if std::thread::spawn(move || {
            #[allow(clippy::unwrap_used)]
            let value = rx.recv().unwrap();
            #[allow(clippy::unwrap_used)]
            stdin.write_all(value.as_bytes()).unwrap();
        })
        .join()
        .is_err()
        {
            return Err(Error::FzfSpawn);
        }

        let output = child.wait_with_output()?;

        println!("{}", String::from_utf8(output.stdout)?);
    }

    Ok(())
}

/// Returnns a `String` containing every glyph and its corresponding name.
fn get_glyph_name_list(font_path: &Path) -> Result<String, Error> {
    let file_contents = fs::read(font_path)?;
    let face = Face::parse(&file_contents, 0)?;
    let glyphs = get_glyphs(&face);
    Ok(get_glyph_names(&face, &glyphs)
        .par_iter()
        .map(|(a, b)| {
            let mut s = String::new();
            s.push(*a);
            s.push(' ');
            s.push_str(b);
            s.push('\n');
            s
        })
        .collect::<String>())
}

/// Returns a `Vector` of all valid glyphs in a face.
#[must_use]
fn get_glyphs(face: &Face) -> Vec<char> {
    let mut s = Vec::<char>::with_capacity(10000);

    face.tables()
        .cmap
        .into_iter()
        .flat_map(|cmap| cmap.subtables)
        .filter(Subtable::is_unicode)
        .filter(|subtable| {
            subtable.platform_id == ttf_parser::PlatformId::Unicode
        })
        .filter(|subtable| {
            matches!(
                subtable.format,
                ttf_parser::cmap::Format::SegmentedCoverage(_)
            )
        })
        .for_each(|subtable| {
            subtable.codepoints(|codepoint| {
                if let Some(unicode_char) = char::from_u32(codepoint) {
                    s.push(unicode_char);
                }
            });
        });

    s
}

/// Returns a `Vector` of glyphs and their corresponding names.
/// Parallelizing is important here.
#[must_use]
fn get_glyph_names(face: &Face, glyphs: &[char]) -> Vec<(char, String)> {
    glyphs
        .par_iter()
        .map(|g| {
            let id = face.glyph_index(*g).unwrap_or_default();
            let glyph_name = face.glyph_name(id).unwrap_or("fake").to_owned();
            (*g, glyph_name)
        })
        .filter(|line| !line.1.starts_with("uni"))
        .collect::<Vec<(char, String)>>()
}

#[derive(Debug, Clone, Copy)]
enum Terminal {
    Ghostty,
    WezTerm,
}

impl Terminal {
    fn get_terminal_font_file(self) -> Result<PathBuf, Error> {
        match self {
            Self::WezTerm => Self::get_wezterm_font_file(),
            Self::Ghostty => Self::get_ghostty_font_file(),
        }
    }

    /// Uses environment variables to get the font file detected for the
    /// current terminal.
    fn identify_terminal() -> Result<Self, Error> {
        if let Ok(term_program) = env::var("TERM_PROGRAM")
            && term_program == "WezTerm"
        {
            return Ok(Self::WezTerm);
        }

        if let Ok(term) = env::var("TERM")
            && term.as_str() == "xterm-ghostty"
        {
            return Ok(Self::Ghostty);
        }

        Err(Error::TerminalNotIdentified)
    }

    fn get_wezterm_font_file() -> Result<PathBuf, Error> {
        let path: PathBuf = String::from_utf8(
            Command::new("wezterm")
                .args(["ls-fonts", "--text", "''"])
                .output()?
                .stdout,
        )?
        .split('\n')
        .flat_map(|line| line.split(' '))
        .filter(|hunk| hunk.contains(".ttf"))
        .map(|hunk| hunk.replace(',', ""))
        .collect::<Vec<String>>()
        .first()
        .ok_or(Error::FontFileNotObtained)?
        .into();

        if !path.is_absolute()
            || path.extension().ok_or(Error::FontFileNotObtained)? != "ttf"
        {
            return Err(Error::FontFileNotObtained);
        }

        Ok(path)
    }

    fn get_ghostty_font_file() -> Result<PathBuf, Error> {
        let mut font_name = String::from_utf8(
            Command::new("ghostty")
                .args(["+show-face", "--string=''"])
                .output()?
                .stdout,
        )?
        .split('\n')
        .filter(|line| line.contains("U+F2B4"))
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .first()
        .ok_or(Error::FontFileNotObtained)?
        .to_owned();

        let i = font_name.find('“').ok_or(Error::FontFileNotObtained)?;
        let mut end_of_line = font_name.split_off(i);
        end_of_line.remove(0);

        let i = end_of_line.find('”').ok_or(Error::FontFileNotObtained)?;
        let _ = end_of_line.split_off(i);

        Self::get_font_file_by_name(&end_of_line)
    }

    fn get_font_file_by_name(name: &str) -> Result<PathBuf, Error> {
        let path: PathBuf = String::from_utf8(
            Command::new("sh")
                .arg("-c")
                .arg(
                    Dependency::FcMatch.exe()?
                        + " -f '%{file}\n' '"
                        + name
                        + "'",
                )
                .output()?
                .stdout,
        )?
        .split('\n')
        .filter(|line| line.contains(".ttf"))
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .first()
        .ok_or(Error::FontFileNotObtained)?
        .into();

        if !path.is_absolute()
            || path.extension().ok_or(Error::FontFileNotObtained)? != "ttf"
        {
            return Err(Error::FontFileNotObtained);
        }

        Ok(path)
    }
}

impl std::fmt::Display for Terminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Ghostty => write!(f, "Ghostty"),
            Self::WezTerm => write!(f, "WezTerm"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Dependency {
    FcMatch,
    Fzf,
}

impl Dependency {
    fn exe(self) -> Result<String, Error> {
        let exe_name = match self {
            Self::FcMatch => "fc-match",
            Self::Fzf => "fzf",
        }
        .to_string();

        if Command::new(&exe_name).arg("--version").output().is_err() {
            return Err(Error::RuntimeDependencyNotFound(self));
        }

        Ok(exe_name)
    }
}

#[derive(Debug)]
pub enum Error {
    FaceParsing(ttf_parser::FaceParsingError),
    FontFileNotObtained,
    FromUtf8Error(std::string::FromUtf8Error),
    FzfSpawn,
    GlyphNotFound,
    InvalidUnicodeConversion,
    Io(std::io::Error),
    Recv(std::sync::mpsc::RecvError),
    RuntimeDependencyNotFound(Dependency),
    Send,
    TerminalNotIdentified,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::FaceParsing(ref err) => {
                write!(f, "ttf_parser face parsing error: {err}")
            }
            Self::FontFileNotObtained => {
                write!(f, "Could not determine font file error")
            }
            Self::FromUtf8Error(ref err) => {
                write!(f, "STD string from utf8 error: {err}")
            }
            Self::FzfSpawn => {
                write!(f, "Fzf spawning error")
            }
            Self::GlyphNotFound => {
                write!(f, "Glyph not found in face error")
            }
            Self::InvalidUnicodeConversion => write!(
                f,
                "Number could not be converted to unicode character error"
            ),
            Self::Io(ref err) => write!(f, "STD IO error: {err}"),
            Self::Recv(ref err) => {
                write!(f, "Reciever error: {err}")
            }
            Self::RuntimeDependencyNotFound(dep) => {
                write!(f, "Runtime dependency not found error: {dep:?}")
            }
            Self::Send => {
                write!(f, "Thread send error")
            }
            Self::TerminalNotIdentified => {
                write!(
                    f,
                    "Could not identify terminal program error. \
                    Consider specifying the font manually with '-f'"
                )
            }
        }
    }
}

impl From<ttf_parser::FaceParsingError> for Error {
    fn from(value: ttf_parser::FaceParsingError) -> Self {
        Self::FaceParsing(value)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::FromUtf8Error(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(value: std::sync::mpsc::RecvError) -> Self {
        Self::Recv(value)
    }
}

pub mod helper {
    use super::{Error, Face, GlyphId, Subtable};

    /// Print out subtables for discovering differences
    pub fn print_subtables(face: &Face) {
        for (i, subtable) in face
            .tables()
            .cmap
            .iter()
            .flat_map(|cmap| cmap.subtables)
            .enumerate()
        {
            println!("COUNT: {i}");
            println!("ENCODING ID: {}", subtable.encoding_id);
            println!("FORMAT: {:?}", subtable.format);
            println!("PLATFORM ID: {:?}", subtable.platform_id);
            println!("IS UNICODE: {:?}", subtable.is_unicode());

            subtable.codepoints(|codepoint| {
                if let Some(unicode_char) = std::char::from_u32(codepoint) {
                    let id: GlyphId =
                        face.glyph_index(unicode_char).unwrap_or_default();
                    let name = face.glyph_name(id).unwrap_or("fake");
                    let mut line = String::new();
                    line.push(unicode_char);
                    line.push(' ');
                    line.push_str(name);
                    println!("{line}");
                }
            });
        }
    }

    /// Get the unicode character corresponding to a named glyph in a font.
    ///
    /// # Errors
    ///
    /// `name` does not match any glyph.
    pub fn get_unicode_char(face: &Face, name: &str) -> Result<char, Error> {
        let glyph_id =
            face.glyph_index_by_name(name).ok_or(Error::GlyphNotFound)?;
        let mut unicode_num: u32 = 0;

        face.tables()
            .cmap
            .into_iter()
            .flat_map(|cmap| cmap.subtables)
            .filter(Subtable::is_unicode)
            .for_each(|subtable| {
                subtable.codepoints(|codepoint| {
                    if char::from_u32(codepoint)
                        .and_then(|ch| face.glyph_index(ch))
                        == Some(glyph_id)
                    {
                        unicode_num = codepoint;
                    }
                });
            });

        let Some(unicode_char) = std::char::from_u32(unicode_num) else {
            return Err(Error::InvalidUnicodeConversion);
        };

        Ok(unicode_char)
    }
}
