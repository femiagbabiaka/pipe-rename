use ansi_term::Colour;
use clap::App;

use dialoguer::Confirm;
use std::env;
use std::fmt::Display;
use std::fs;
use std::io::SeekFrom;
use std::io::{self, Read, Seek, Write};
use std::process::Command;
use std::process::Stdio;

use thiserror::Error;

#[derive(Debug, Clone)]
struct Rename {
    original: String,
    new: String,
}

impl Display for Rename {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.original, self.new)
    }
}

#[derive(Error, Debug, Clone)]
pub enum RenamerError {
    #[error("No replacements found")]
    NoReplacementsFound,
    #[error("Unequal number of files")]
    UnequalLines,
}

fn find_renames(old_lines: String, new_lines: String) -> Result<Vec<Rename>, RenamerError> {
    let old_lines = old_lines.lines();
    let new_lines = new_lines.lines();
    if old_lines.clone().count() != new_lines.clone().count() {
        return Err(RenamerError::UnequalLines);
    }
    let renames: Vec<Rename> = old_lines
        .zip(new_lines)
        .filter_map(|(old, new)| {
            if old.eq(new) {
                None
            } else {
                Some(Rename {
                    original: old.to_string(),
                    new: new.to_string(),
                })
            }
        })
        .collect();

    Ok(renames)
}

fn prim() -> anyhow::Result<&'static str> {
    let _matches = App::new("renamer")
                          .version("1.0")
                          .author("Marcus B. <me@mbuffett.com")
                          .about("Takes a list of files and renames/removes them, by piping them through an external editor")
                          .get_matches();
    let input = {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };
    let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
    {
        // write!(tmpfile, "{}", input);
        write!(tmpfile, "{}", input)?;
        // .write_all(input.as_bytes())
        // .expect("Failed to write to stdin");
        let editor = env::var("EDITOR").unwrap_or("vim".to_string());
        tmpfile.seek(SeekFrom::Start(0)).unwrap();
        let child = Command::new(editor)
            .arg(tmpfile.path().to_str().unwrap().to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .spawn()
            .expect("failed to execute process");

        child.wait_with_output()?;
    }
    let old_lines = input;
    let new_lines = fs::read_to_string(tmpfile)?;
    let replacements = find_renames(old_lines, new_lines);
    if let Ok(replacements) = replacements.clone() {
        if replacements.is_empty() {
            println!("No replacements found");
            return Err(RenamerError::NoReplacementsFound.into());
        }
        println!();
        println!(
            "{}",
            Colour::Yellow.paint("The following replacements were found")
        );
        println!();
        for replacement in replacements.clone() {
            println!("{}", Colour::Green.paint(replacement.to_string()));
        }
        println!();
        if Confirm::new()
            .with_prompt("Execute these renames?")
            .interact()?
        {
            for replacement in replacements {
                fs::rename(replacement.original, replacement.new)?; // Rename a.txt to b.txt
            }
        } else {
            println!("Aborting")
        }
    }
    if let Err(err) = replacements {
        println!("{}", err);
    }
    Ok("")
}

fn main() {
    prim();
}