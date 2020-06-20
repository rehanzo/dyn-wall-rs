/*
   dyn-wall-rs 1.1.2
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
use crate::errors::{ConfigFileErrors, Errors};
use clap::AppSettings;
use dirs::config_dir;
use dyn_wall_rs::{print_schedule, sorted_dir_iter, time_track::Time, wallpaper_listener};
use std::fs::canonicalize;
use std::{
    error::Error, fs::create_dir_all, fs::File, io::Read, io::Write, str::FromStr, sync::Arc,
};
use structopt::StructOpt;
use walkdir::WalkDir;

pub mod errors;
pub mod time_track;

#[derive(StructOpt)]
#[structopt(
    about = "Helps user set a dynamic wallpaper and lockscreen. Make sure the wallpapers are named in numerical order based on the order you want. For more info and help, go to https://github.com/RAR27/dyn-wall-rs",
    author = "Rehan Rana <rehanalirana@tuta.io>"
)]
struct Args {
    #[structopt(
        short,
        long,
        value_name = "DIRECTORY",
        help = "Sets the wallpaper based on the current time and changes the wallpaper throughout the day based on the time",
        conflicts_with = "Schedule"
    )]
    auto: Option<String>,

    #[structopt(
        short,
        long,
        value_name = "DIRECTORY",
        help = r#"Changes wallpapers based on custom times set through a config file created at ~/.config/dyn-wall-rs/config for Unix systems and C:\Users\<USER NAME>\AppData\Roaming\dyn-wall-rs for Windows systems"#
    )]
    custom: Option<String>,

    #[structopt(
        short = "l",
        long = "lockscreen",
        value_name = "COMMAND",
        help = r#"Sends image as argument to command specified. Use alongside listener or custom. If the command itself contains arguments, wrap in quotation ex. dyn-wall-rs -a /path/to/dir -l "betterlockscreen -u""#
    )]
    prog: Option<String>,

    #[structopt(
        short,
        long,
        value_name = "COMMAND",
        help = r#"Sends image as argument to command specified. Use alongside listener or custom. If the command itself contains arguments, wrap in quotation ex. dyn-wall-rs -a /path/to/dir -l "betterlockscreen -u""#
    )]
    schedule: Option<String>,

    #[structopt(
        short,
        long,
        value_name = "BACKEND",
        help = "Uses the specified method as a backend"
    )]
    backend: Option<String>,
}

fn main() {
    //convert to clap to add setting to print help message if no argument sent
    //and make help message order same as Args struct order
    let clap = Args::clap()
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DeriveDisplayOrder);
    let args = Args::from_clap(&clap.get_matches());
    let mut program = Arc::new(None);
    let backend = Arc::new(args.backend);

    if let Some(prog) = args.prog {
        if args.auto.is_none() && args.custom.is_none() {
            eprintln!("This option is to be used along with auto or custom");
        } else {
            program = Arc::new(Some(String::from(prog)));
        }
    }

    if let Some(dir) = args.auto {
        let dir = dir.as_str();
        let dir_count = WalkDir::new(dir).into_iter().count() - 1;

        if 1440 % dir_count != 0 || dir_count == 0 {
            eprintln!("{}", Errors::CountCompatError(dir_count));
        } else {
            match check_dir_exists(dir) {
                Err(e) => eprintln!("{}", e),
                Ok(_) => {
                    let dir = canonicalize(dir).unwrap();
                    let dir = dir.to_str().unwrap();
                    if let Err(e) = wallpaper_listener(
                        String::from(dir),
                        dir_count,
                        Arc::clone(&program),
                        None,
                        Arc::clone(&backend),
                    ) {
                        eprintln!("{}", e);
                    }
                }
            }
        }
    }

    if let Some(dir) = args.schedule {
        let dir = dir.as_str();
        let dir_count = WalkDir::new(dir).into_iter().count() - 1;

        if 1440 % dir_count != 0 || dir_count == 0 {
            eprintln!("{}", Errors::CountCompatError(dir_count));
        } else {
            match check_dir_exists(dir) {
                Err(e) => eprintln!("{}", e),
                Ok(_) => {
                    let dir = canonicalize(dir).unwrap();
                    let dir = dir.to_str().unwrap();
                    if let Err(e) = print_schedule(dir, dir_count) {
                        eprintln!("{}", e);
                    }
                }
            }
        }
    }

    if let Some(dir) = args.custom {
        let dir = dir.as_str();
        let dir_count = WalkDir::new(dir).into_iter().count() - 1;

        match config_parse() {
            Err(e) => {
                eprintln!("{}", e);
            }
            Ok(times) => match check_dir_exists(dir) {
                Err(e) => eprintln!("{}", e),
                Ok(_) => {
                    let dir = canonicalize(dir).unwrap();
                    let dir = dir.to_str().unwrap();
                    if let Err(e) = wallpaper_listener(
                        String::from(dir),
                        dir_count,
                        Arc::clone(&program),
                        Some(times),
                        Arc::clone(&backend),
                    ) {
                        eprintln!("{}", e);
                    }
                }
            },
        }
    }
}

fn config_parse() -> Result<Vec<Time>, Box<dyn Error>> {
    let mut times = Vec::new();
    let file = File::open(format!(
        "{}/dyn-wall-rs/config",
        config_dir()
            .ok_or_else(|| Errors::ConfigFileError(ConfigFileErrors::NotFound))?
            .to_str()
            .unwrap()
    ))
    .map_err(|_| Errors::ConfigFileError(ConfigFileErrors::NotFound));

    let mut file = match file {
        Ok(s) => Ok(s),
        Err(e) => {
            create_config()?;
            Err(e)
        }
    }?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let contents_split = contents.lines();

    for line in contents_split {
        if line.starts_with('#') {
            continue;
        }

        for time in line.split_whitespace() {
            times.push(Time::from_str(time)?);
        }
    }
    Ok(times)
}

fn create_config() -> Result<(), Box<dyn Error>> {
    let config_dir =
        config_dir().ok_or_else(|| Errors::ConfigFileError(ConfigFileErrors::NotFound))?;
    create_dir_all(format!("{}/dyn-wall-rs", config_dir.to_str().unwrap()))?;
    let mut config_file = File::create(format!(
        "{}/dyn-wall-rs/config",
        config_dir.to_str().unwrap()
    ))?;
    let default_test = "# Write down the times at which you want the wallpaper to change below\n\
    # The times must be in chronological order\n\
    # The number of images and the number of times should be equal\n\
    # ex:\n\
    # 00:00\n\
    # 02:00\n\
    # 04:00\n\
    # 06:00\n\
    # 08:00\n\
    # 10:00\n\
    # 12:00\n\
    # 14:00\n\
    # 16:00\n\
    # 18:00\n\
    # 20:00\n\
    # 22:00\n\
    # The times are linked to the files in numerical order. This means that in the example above,\n\
    # 1.png will be your wallpaper at 00:00, 2.png will be your wallpaper at 02:00, etc.\n\
    # The directory would need 12 images for this example to work, since there are 12 times stated";

    config_file.write_all(default_test.as_bytes())?;
    Ok(())
}

fn check_dir_exists(dir: &str) -> Result<(), Errors> {
    let mut dir_iter = sorted_dir_iter(dir);

    if dir_iter.next().unwrap().is_err() {
        Err(Errors::DirNonExistantError(dir.to_string()))
    } else {
        Ok(())
    }
}
