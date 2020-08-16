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
use dyn_wall_rs::{
    check_dir_exists,
    config::{Args, Times},
    listener_setup, print_schedule, sun_timings,
    time_track::Time,
    wallpaper_listener,
};
use std::{
    error::Error, fs::canonicalize, fs::create_dir_all, fs::File, io::Read, io::Write, process,
    str::FromStr, sync::Arc,
};
use structopt::StructOpt;
use walkdir::WalkDir;

pub mod config;
pub mod errors;
pub mod time_track;

fn main() {
    //convert to clap to add setting to print help message if no argument sent
    //and make help message order same as Args struct order
    let clap = Args::clap().setting(AppSettings::DeriveDisplayOrder);
    let cli_args = Args::from_clap(&clap.get_matches());
    let cli_args_used = !(Args::default() == cli_args);
    let mut args: Args;
    //min depth of what files should be looked at, will remain as 1 if not syncing with sun, will
    //change to 2 if syncing with sun to ignore the directory names, focusing just on the files
    let mut min_depth = 1;

    //pulling from config file if cli arguments are not specified, or if just custom timings were
    //specified
    match config_parse(cli_args_used) {
        Err(e) => {
            eprint!("{}", e);
            process::exit(1);
        }
        Ok(s) => {
            //rust doesn't let you assign when deconstructing, so this workaround is required
            let (temp_times, config_args) = s;

            args = Args {
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
                eprintln!("Directory not specified");
            }
            //if latitude is specified, then longitude and elevation is required as well, so we
            //just need to check for one of them
            else if let Some(lat) = args.lat {
                if args.long.is_none() {
                    eprintln!("Error: lat needs to be specified with long");
                    process::exit(1);
                } else {
                    let dir = args.directory.to_owned();
                    match dir {
                        None => eprintln!("Error: Directory needs to be specified"),
                        Some(dir) => {
                            let dir = dir.as_str();
                            match sun_timings(
                                dir,
                                lat,
                                args.long.unwrap(),
                                args.elevation.or_else(|| Some(0.0)).unwrap(),
                            ) {
                                Err(e) => eprintln!("Error: {}", e),
                                Ok(s) => {
                                    args.times = Some(s);
                                    min_depth = 2;
                                }
                            }
                        }
                    }
                }
            } else if args.long.is_some() {
                eprintln!("Error: long neds to be specified with lat");
                process::exit(1);
            }
        }
    }

    //handle custom programs specified by user
    if args.programs.is_some() {
        if args.directory.is_none() && !args.schedule {
            eprintln!("Error: The program option is to be used with a specified directory");
        }
    }

    //handle custom backend specified by user
    if args.backend.is_some() {
        if args.directory.is_none() {
            eprintln!("Error: The backend option is to be used with a specified directory");
        }
    }

    if let Some(dir) = &args.directory {
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
                if args.times.is_none() {
                    if 1440 % dir_count != 0 || dir_count == 0 {
                        eprintln!("{}", Errors::CountCompatError(dir_count));
                    } else {
                        let (_, step_time, mut loop_time, _) = listener_setup(dir);
                        match step_time {
                            Err(e) => eprintln!("{}", e),
                            Ok(step_time) => {
                                let mut times: Vec<Time> = vec![];
                                for _ in 1..=dir_count {
                                    times.push(loop_time);
                                    loop_time += step_time;
                                }
                                args.times = Some(times);
                            }
                        }
                    }
                }
            }
        }
        let args_arc = Arc::new(args);
        if args_arc.schedule {
            if let Err(e) = print_schedule(dir, min_depth, Arc::clone(&args_arc)) {
                eprintln!("{}", e);
            }
        } else if let Err(e) =
            wallpaper_listener(String::from(dir), Arc::clone(&args_arc), min_depth)
        {
            eprintln!("{}", e);
        }
    } else if args.schedule {
        eprintln!("Error: The schedule option is to be used alongside a specified directory");
    }
}

//parse config file
fn config_parse(cli_args_used: bool) -> Result<(Option<Vec<Time>>, Args), Box<dyn Error>> {
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
