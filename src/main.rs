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
use serde::{Deserialize, Serialize};
use std::fs::canonicalize;
use std::process;
use std::{
    error::Error, fs::create_dir_all, fs::File, io::Read, io::Write, str::FromStr, sync::Arc,
};
use structopt::StructOpt;
use toml;
use walkdir::WalkDir;

pub mod errors;
pub mod time_track;

#[derive(StructOpt)]
#[structopt(
    about = "Helps user set a dynamic wallpaper and lockscreen. Make sure the wallpapers are named in numerical order based on the order you want. For more info and help, go to https://github.com/RAR27/dyn-wall-rs",
    author = "Rehan Rana <rehanalirana@tuta.io>"
)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Args {
    #[structopt(
        short,
        long,
        value_name = "DIRECTORY",
        help = "Sets the wallpaper based on the current time and changes the wallpaper throughout the day. The wallpaper will change based on the user specified times within the config file or, if custom timings are not set, it will automatically divide the wallpapers into equal parts throughout the day.",
        conflicts_with = "Schedule"
    )]
    directory: Option<String>,

    #[structopt(
        short = "p",
        long = "program",
        value_name = "COMMAND",
        help = r#"Sends image as argument to command specified. Use alongside listener or custom. If the command itself contains arguments, wrap in quotation ex. dyn-wall-rs -a /path/to/dir -l "betterlockscreen -u"
If arguments after wallpaper argument are needed, use !WALL as a placeholder for wallpaper argument, and add rest of arguments ex. dyn-wall-rs -a /path/to/dir -p "betterlockscreen -u !WALL -b 1""#
    )]
    program: Option<String>,

    #[structopt(
        short,
        long,
        value_name = "COMMAND",
        help = "Will present you with a schedule of when your wallpaper will change if you have not set custom times in the config file"
    )]
    schedule: Option<String>,

    #[structopt(
        short,
        long,
        value_name = "BACKEND",
        help = "Uses the specified method as the backend to change the wallpaper"
    )]
    backend: Option<String>,
}
#[derive(Deserialize, Serialize)]
struct Times {
    times: Option<Vec<String>>,
}

fn main() {
    //convert to clap to add setting to print help message if no argument sent
    //and make help message order same as Args struct order
    let clap = Args::clap().setting(AppSettings::DeriveDisplayOrder);
    let mut args = Args::from_clap(&clap.get_matches());
    let mut program = Arc::new(None);
    let mut backend = Arc::new(None);
    let cli_args = !((Args {
        directory: None,
        program: None,
        schedule: None,
        backend: None,
    }) == args);
    let mut times: Vec<Time> = vec![];

    match config_parse(cli_args) {
        Err(e) => {
            eprint!("{}", e);
            process::exit(1);
        }
        Ok(s) => {
            //rust doesn't let you assign when deconstructing, so this workaround is required
            let (temp_times, temp_args) = s;
            if let Some(s) = temp_times {
                times = s;
            }
            if !cli_args {
                args = temp_args;
            }
        }
    }

    if let Some(prog) = args.program {
        if args.directory.is_none() {
            eprintln!(
                "Specifying a program is to be used along with the specification of a directory"
            );
        } else {
            program = Arc::new(Some(String::from(prog)));
        }
    }

    if let Some(back) = args.backend {
        backend = Arc::new(Some(back));
        if args.directory.is_none() {
            eprintln!("The backend option is to be used with a specified directory");
        }
    }

    if let Some(dir) = args.directory {
        let dir = dir.as_str();
        let dir_count = WalkDir::new(dir).into_iter().count() - 1;

        match check_dir_exists(dir) {
            Err(e) => eprintln!("{}", e),
            Ok(_) => {
                if times.len() == 0 {
                    if 1440 % dir_count != 0 || dir_count == 0 {
                        eprintln!("{}", Errors::CountCompatError(dir_count));
                    } else {
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
                } else {
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
}

fn config_parse(cli_args: bool) -> Result<(Option<Vec<Time>>, Args), Box<dyn Error>> {
    let file = File::open(format!(
        "{}/dyn-wall-rs/config.toml",
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

    if !cli_args {
        let mut empty = true;
        for line in contents.lines() {
            if !line.contains("#") {
                empty = false;
            }
        }
        if empty {
            return Err(Errors::ConfigFileError(ConfigFileErrors::Empty).into());
        }
    }

    let args_string = toml::from_str(contents.as_str());
    let args_string: Args = match args_string {
        Err(e) => {
            return Err(Errors::ConfigFileError(ConfigFileErrors::Other(e.to_string())).into());
        }
        Ok(s) => s,
    };

    let times_string = toml::from_str(contents.as_str());
    let times_string: Times = match times_string {
        Err(e) => {
            return Err(Errors::ConfigFileError(ConfigFileErrors::Other(e.to_string())).into());
        }
        Ok(s) => s,
    };

    match times_string.times {
        None => Ok((None, args_string)),
        Some(s) => {
            let times: Result<Vec<_>, _> = s.iter().map(|time| Time::from_str(time)).collect();
            let times = times?;
            Ok((Some(times), args_string))
        }
    }
}

fn create_config() -> Result<(), Box<dyn Error>> {
    let config_dir =
        config_dir().ok_or_else(|| Errors::ConfigFileError(ConfigFileErrors::NotFound))?;
    create_dir_all(format!("{}/dyn-wall-rs", config_dir.to_str().unwrap()))?;
    let mut config_file = File::create(format!(
        "{}/dyn-wall-rs/config.toml",
        config_dir.to_str().unwrap()
    ))?;
    let default_test = r#"# Write down the times at which you want the wallpaper to change below
    # The times must be in chronological order
    # The number of images and the number of times should be equal
    #
    # ex:
    # times = [
    #   "00:00",
    #   "02:00",
    #   "04:00",
    #   "06:00",
    #   "08:00",
    #   "10:00",
    #   "12:00",
    #   "14:00",
    #   "16:00",
    #   "18:00",
    #   "20:00",
    #   "22:00",
    # ]
    #
    # The times are linked to the files in numerical order. This means that in the example above,
    # 1.png will be your wallpaper at 00:00, 2.png will be your wallpaper at 02:00, etc.
    # The directory would need 12 images for this example to work, since there are 12 times stated
    # Other config options are stated below; uncomment them and fill them as you would from the command line.
    #times = []
    #directory = "/path/to/dir"
    #backend = "backend"
    #program = "command""#;

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
