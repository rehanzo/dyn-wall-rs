/*
   dyn-wall-rs 2.0.2
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
use dyn_wall_rs::{print_schedule, sun_timings, time_track::Time, wallpaper_listener, listener_setup};
use serde::{Deserialize, Serialize};
use std::{
    error::Error, fs::create_dir_all, fs::File, io::Read, io::Write, str::FromStr, sync::Arc, process, fs::canonicalize,
};
use structopt::StructOpt;
use toml;
use walkdir::WalkDir;

pub mod errors;
pub mod time_track;

#[derive(StructOpt, Default)]
#[structopt(
    about = "Helps user set a dynamic wallpaper and lockscreen. Make sure the wallpapers are named in numerical order based on the order you want. For more info and help, go to https://github.com/RAR27/dyn-wall-rs",
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
        long = "programs",
        value_name = "COMMAND",
        help = r#"Sends image as argument to command specified. Use alongside listener or custom. If the command itself contains arguments, wrap in quotation ex. dyn-wall-rs -a /path/to/dir -l "betterlockscreen -u""#
    )]
    program: Option<Vec<String>>,

    #[structopt(
        short,
        long,
        value_name = "DIRECTORY",
        help = "Will present you with a schedule of when your wallpaper will change if you have not set custom times in the config file",
        takes_value = false,
    )]
    #[serde(skip)]
    schedule: bool,

    #[structopt(
        short,
        long,
        value_name = "BACKEND",
        help = "Uses the specified method as the backend to change the wallpaper. Custom command can be used"
    )]
    backend: Option<String>,

    #[structopt(
        long,
        value_name = "LATITUDE",
        help = "Latitude of current location. Requires the use of the long and elevation options as well",
        requires_all = &["long", "elevation"],
        allow_hyphen_values(true)
    )]
    lat: Option<f64>,

    #[structopt(
        long,
        value_name = "LONGITUDE",
        help = "Longitude of current location. Requires the use of the lat and elevation options as well",
        requires_all = &["lat", "elevation"],
        allow_hyphen_values(true)
    )]
    long: Option<f64>,

    #[structopt(
        long,
        value_name = "ELEVATION",
        help = "Elevation of current location. Requires the use of the lat and long options as well",
        requires_all = &["lat", "long"],
        allow_hyphen_values(true)
    )]
    elevation: Option<f64>,
}

//not optimal, but it seems serde can really only work on structs. Would be great if I could
//serialize straight into a vector, but it doesn't seem like I can, so this is a workaround
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
    let cli_args = !(Args::default() == args);
    let mut times: Vec<Time> = vec![];
    //min depth of what files should be looked at, will remain as 1 if not syncing with sun, will
    //change to 2 if syncing with sun to ignore the directory names, focusing just on the files
    let mut min_depth = 1;

    //pulling from config file if cli arguments are not specified, or if just custom timings were
    //specified
    match config_parse(cli_args) {
        Err(e) => {
            eprint!("{}", e);
            process::exit(1);
        }
        Ok(s) => {
            //rust doesn't let you assign when deconstructing, so this workaround is required
            let (temp_times, temp_args) = s;

            if !cli_args {
                args = temp_args;
                //the default is all fields none, this is fine becuase if other options are used by
                //themselves, specific errors come up.
                if Args::default() == args {
                    eprintln!("Directory not specified");
                }
            }

            //for custom timings
            if let Some(s) = temp_times {
                times = s;
            }

            //if latitude is specified, then longitude and elevaiton is required as well, so we
            //just need to check for one of them
            else if let Some(lat) = args.lat {
                let dir = args.directory.to_owned();
                match dir {
                    None => eprintln!("Directory needs to be specified"),
                    Some(dir) => {
                        let dir = dir.as_str();
                        let dir_night = format!("{}/night", dir);
                        let dir_night = dir_night.as_str();
                        let dir_day = format!("{}/day", dir);
                        let dir_day = dir_day.as_str();

                        //checking if the directories exist
                        if check_dir_exists(dir).is_err() {
                            eprintln!("{}", Errors::FilePathError);
                            process::exit(1);
                        } else if check_dir_exists(dir_night).is_err()
                            || check_dir_exists(dir_day).is_err()
                        {
                            eprintln!("Error: Make sure night and day directories are created within master directory");
                            process::exit(1);
                        } else {
                            //now we know directories exist, so lets get the counts of the night
                            //and day directories and send it to sun_timings function to get vector
                            //of times based on sunset and sunrise
                            let dir_count_night = WalkDir::new(dir_night)
                                .min_depth(min_depth)
                                .into_iter()
                                .count();
                            let dir_count_day = WalkDir::new(dir_day)
                                .min_depth(min_depth)
                                .into_iter()
                                .count();
                            times = sun_timings(
                                lat,
                                args.long.unwrap(),
                                args.elevation.unwrap(),
                                dir_count_day as u32,
                                dir_count_night as u32,
                            );
                            min_depth = 2;
                        }
                    }
                }
            }
        }
    }

    //handle custom programs specified by user
    if let Some(progs) = args.program {
        if args.directory.is_none() {
            eprintln!("Error: The program option is to be used with a specified directory");
        } else {
            program = Arc::new(Some(progs));
        }
    }

    //handle custom backend specified by user
    if let Some(back) = args.backend {
        backend = Arc::new(Some(back));
        if args.directory.is_none() {
            eprintln!("Error: The backend option is to be used with a specified directory");
        }
    }

    if let Some(dir) = args.directory {
        let dir = dir.as_str();
        let dir_count = WalkDir::new(dir).min_depth(min_depth).into_iter().count();
        let dir = canonicalize(dir).expect("Failed to canonicalize");
        let dir = dir.to_str().expect("Couldn't convert to string");

        match check_dir_exists(dir) {
            Err(e) => eprintln!("{}", e),
            Ok(_) => {
                //if the times vector is empty, that means that user didn't specify, so we have to
                //send "None" to wallpaper listener, which will create a evenly spread timings
                //vector
                if times.len() == 0 {
                    if 1440 % dir_count != 0 || dir_count == 0 {
                        eprintln!("{}", Errors::CountCompatError(dir_count));
                    }
                    else {
                        let (_, step_time, mut loop_time, _) = listener_setup(dir);
                        match step_time {
                            Err(e) => eprintln!("{}", e),
                            Ok(step_time) => {
                                for _ in 1..=dir_count {
                                    times.push(loop_time);
                                    loop_time += step_time;
                                }
                            }
                        }
                    }
                }
            }
        }
        if args.schedule {
            if let Err(e) = print_schedule(dir, min_depth, &times) {
                eprintln!("{}", e);
            }
        }
        else if let Err(e) = wallpaper_listener(
            String::from(dir),
            Arc::clone(&program),
            times.clone(),
            Arc::clone(&backend),
            min_depth,
        ) {
            eprintln!("{}", e);
        }

    }
}

//parse config file
fn config_parse(cli_args: bool) -> Result<(Option<Vec<Time>>, Args), Box<dyn Error>> {
    let file = File::open(format!(
        "{}/dyn-wall-rs/config.toml",
        config_dir()
            .ok_or_else(|| Errors::ConfigFileError(ConfigFileErrors::NotFound))?
            .to_str()
            .unwrap()
    ))
    .map_err(|_| Errors::ConfigFileError(ConfigFileErrors::NotFound));

    let file = match file {
        Ok(s) => Ok(s),
        Err(e) => {
            create_config()?;
            Err(e)
        }
    };

    if file.is_err() && cli_args {
        println!("A config file has been created");
        return Ok((None, Args::default()));
    }
    let mut file = file.unwrap();

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
            //provide our own error if empty, rather than less descriptive error from serde
            return Err(Errors::ConfigFileError(ConfigFileErrors::Empty).into());
        }
    }

    let args_string = toml::from_str(contents.as_str());
    let args_serialized: Args = match args_string {
        Err(e) => {
            return Err(Errors::ConfigFileError(ConfigFileErrors::Other(e.to_string())).into());
        }
        Ok(s) => s,
    };

    let times_string = toml::from_str(contents.as_str());
    let times_serialized: Times = match times_string {
        Err(e) => {
            return Err(Errors::ConfigFileError(ConfigFileErrors::Other(e.to_string())).into());
        }
        Ok(s) => s,
    };

    match times_serialized.times {
        None => Ok((None, args_serialized)),
        Some(s) => {
            let times: Result<Vec<_>, _> = s.iter().map(|time| Time::from_str(time)).collect();
            let times = times?;
            Ok((Some(times), args_serialized))
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
    let contents = r#"# Type the times at which you want the wallpaper to change as shown in the example below
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
# Config options are stated below; uncomment them and fill them as you would from the command line.
#times = []
#directory = "/path/to/dir"
#backend = "feh"
#program = ["echo test1", "echo test2"]
#lat = 99
#long = -99
#elevation = 99"#;

    config_file.write_all(contents.as_bytes())?;
    Ok(())
}

fn check_dir_exists(dir: &str) -> Result<(), Errors> {
    let mut dir_iter = WalkDir::new(dir).into_iter();

    if dir_iter.next().unwrap().is_err() {
        Err(Errors::DirNonExistantError(dir.to_string()))
    } else {
        Ok(())
    }
}
