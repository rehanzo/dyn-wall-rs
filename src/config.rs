/*
   dyn-wall-rs 2.1.0
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

use crate::{check_dir_exists, sun_timings, ConfigFileErrors, Errors, Time};
use dirs_next::config_dir;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::create_dir_all,
    fs::File,
    io::{Read, Write},
    str::FromStr,
};
use structopt::StructOpt;

#[derive(StructOpt, Default)]
#[structopt(
    about = "Helps user set a dynamic wallpaper and lockscreen. Make sure the wallpapers are named in numerical order based on the order you want. For more info and help, go to https://github.com/RAR27/dyn-wall-rs"
)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Args {
    #[structopt(
        short,
        long,
        value_name = "DIRECTORY",
        help = "Sets the wallpaper based on the current time and changes the wallpaper throughout the day. The wallpaper will change based on the user specified times within the config file or, if custom timings are not set, or if location isn't specified, it will automatically divide the wallpapers into equal parts throughout the day.",
        conflicts_with = "Schedule"
    )]
    pub directory: Option<String>,

    #[structopt(
        short = "p",
        long = "programs",
        value_name = "COMMAND",
        help = r#"Sends image as argument to command specified. Use alongside the directory option. If the command itself contains arguments, wrap in quotation ex. dyn-wall-rs -a /path/to/dir -l "betterlockscreen -u""#
    )]
    pub programs: Option<Vec<String>>,

    #[structopt(
        short,
        long,
        help = "Will present you with a schedule of when your wallpaper will change. To be used alongside the directory option",
        //requires = "directory",
        takes_value = false,
    )]
    #[serde(skip)]
    pub schedule: bool,

    #[structopt(
        short,
        long,
        value_name = "BACKEND",
        help = "Uses the specified method as the backend to change the wallpaper. Custom command can be used"
    )]
    pub backend: Option<String>,

    #[structopt(
        long,
        value_name = "LATITUDE",
        help = "Latitude of current location. Requires the use of the long option",
        //requires_all = &["long", "elevation"],
        allow_hyphen_values(true)
    )]
    pub lat: Option<f64>,

    #[structopt(
        long,
        value_name = "LONGITUDE",
        help = "Longitude of current location. Requires the use of the lat option",
        //requires_all = &["lat", "elevation"],
        allow_hyphen_values(true)
    )]
    pub long: Option<f64>,

    #[structopt(
        long,
        value_name = "ELEVATION",
        help = "Elevation of current location. Optional, but allows for a more accurate calculation of sunrise and sunset times",
        //requires_all = &["lat", "long"],
        allow_hyphen_values(true)
    )]
    pub elevation: Option<f64>,

    #[structopt(skip)]
    #[serde(skip)]
    pub times: Option<Vec<Time>>,
}

//not optimal, but it seems serde can really only work on structs. Would be great if I could
//serialize straight into a vector, but it doesn't seem like I can, so this is a workaround
#[derive(Deserialize, Serialize)]
pub struct Times {
    pub times: Option<Vec<String>>,
}

impl Args {
    pub fn mixed(cli_args: Args, cli_args_used: bool) -> Result<Self, Box<dyn Error>> {
        //rust doesn't let you assign when deconstructing, so this workaround is required
        let (temp_times, config_args) = config_parse(cli_args_used)?;

        let mut args = Args {
            directory: if cli_args.directory.is_some() {
                cli_args.directory
            } else {
                config_args.directory
            },
            programs: if cli_args.programs.is_some() {
                cli_args.programs
            } else {
                config_args.programs
            },
            schedule: cli_args.schedule,
            backend: if cli_args.backend.is_some() {
                cli_args.backend
            } else {
                config_args.backend
            },
            lat: if cli_args.lat.is_some() {
                cli_args.lat
            } else {
                config_args.lat
            },
            long: if cli_args.long.is_some() {
                cli_args.long
            } else {
                config_args.long
            },
            elevation: if cli_args.elevation.is_some() {
                cli_args.elevation
            } else {
                config_args.elevation
            },
            times: temp_times,
        };
        //the default is all fields none, this is fine becuase if other options are used by
        //themselves, specific errors come up.
        if Args::default() == args {
            Err("Directory not specified".into())
        }
        //if latitude is specified, then longitude and elevation is required as well, so we
        //just need to check for one of them
        else if let Some(lat) = args.lat {
            if args.long.is_none() {
                Err("Error: lat needs to be specified with long".into())
            } else {
                let dir = args.directory.to_owned();
                match dir {
                    None => Err("Error: Directory needs to be specified".into()),
                    Some(dir) => {
                        let dir = dir.as_str();
                        match sun_timings(
                            dir,
                            lat,
                            args.long.unwrap(),
                            args.elevation.or_else(|| Some(0.0)).unwrap(),
                        ) {
                            Err(e) => Err(format!("Error: {}", e).into()),
                            Ok(s) => {
                                args.times = Some(s);
                                Ok(args)
                            }
                        }
                    }
                }
            }
        } else if args.long.is_some() {
            Err("Error: long neds to be specified with lat".into())
        }
        //handle custom programs specified by user
        else if args.programs.is_some() && args.directory.is_none() && !args.schedule {
            Err("Error: The program option is to be used with a specified directory".into())
        }
        //handle custom backend specified by user
        else if args.backend.is_some() && args.directory.is_none() {
            Err("Error: The backend option is to be used with a specified directory".into())
        } else if args.schedule && args.directory.is_none() {
            Err("Error: The schedule option is to be used alongside a specified directory".into())
        } else {
            check_dir_exists(&args.directory.to_owned().unwrap())?;
            Ok(args)
        }
    }
}

//parse config file
type UserInput = (Option<Vec<Time>>, Args);
pub fn config_parse(cli_args_used: bool) -> Result<UserInput, Box<dyn Error>> {
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

    if file.is_err() {
        return Err("A config file has been created".into());
    }
    let mut file = file.unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    if !cli_args_used {
        let mut empty = true;
        for line in contents.lines() {
            if !line.contains('#') {
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
