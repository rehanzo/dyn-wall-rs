/*
   dyn-wall-rs 2.1.3
   Rehan Rana <rehanalirana@tuta.io>
   Helps user set a dynamic wallpaper and lockscreen. For more info and help, go to https://github.com/RAR27/dyn-wall-rs
   Copyright (C) 2020  Rehan Rana

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
use dirs_next::config_dir;
use std::{error, fmt};

#[cfg(not(windows))]
const DIR_SLASH: &str = "/";
#[cfg(windows)]
const DIR_SLASH: &str = r#"\"#;

#[derive(Debug)]
///Custom error types
pub enum Errors {
    FilePathError,
    ProgramRunError(String),
    CountCompatError(usize),
    DirNonExistantError(String),
    NoFilesFoundError(String),
    ConfigFileError(ConfigFileErrors),
    BackendNotFoundError(String),
}

#[derive(Debug)]
///Custom error subtypes for ConfigFileError
pub enum ConfigFileErrors {
    Empty,
    FileTimeMismatch,
    FormattingError,
    NotFound,
    OutOfOrder,
    OutOfRange,
    DuplicatesFound,
    Other(String),
}

impl error::Error for Errors {}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Errors::FilePathError => write!(f, "Error while handling file path"),
            Errors::ProgramRunError(prog) => write!(f, "Error while running {}", prog),
            Errors::CountCompatError(count) => {
                match count {
                    0 => {
                        write!(f, "No images found in the given directory")
                    }
                    _ => {
                        write!(f, "Cannot schedule the rotation of {} images evenly throughout the day (the number of images should divide evenly into 1440)", count)
                    }
                }
            }
            Errors::DirNonExistantError(dir) => write!(f, "The directory {} doesn't exist", dir),
            Errors::NoFilesFoundError(loc) => write!(f, "No file(s) found at {}", loc),
            Errors::ConfigFileError(cause) => {
                let template = "Error with config file";
                match cause {
                    ConfigFileErrors::Empty => write!(f, "{}: config file is empty", template),
                    ConfigFileErrors::FileTimeMismatch => write!(f, "{}: the number of times listed in the config file does not equal the number of files in directory", template),
                    ConfigFileErrors::FormattingError => write!(f, "{}: config file not formatted correctly", template),
                    ConfigFileErrors::NotFound => write!(f, "{}: config file not found. One has been created at {}{}dyn-wall-rs{}config.toml for you to edit", template, config_dir().expect("No config directory found").to_str().unwrap(), DIR_SLASH, DIR_SLASH),
                    ConfigFileErrors::OutOfOrder => write!(f, "{}: the order of the times are incorrect", template),
                    ConfigFileErrors::OutOfRange => write!(f, "{}: Custom times should be between 0 - 23:59", template), 
                    ConfigFileErrors::DuplicatesFound => write!(f, "{}: duplicate times found", template),
                    ConfigFileErrors::Other(other_err) => write!(f, "{}: {}", template, other_err),
                }
            }
            Errors::BackendNotFoundError(backend) => write!(f, "Backend '{}' not found", backend),
        }
    }
}
