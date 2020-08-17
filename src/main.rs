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
use dyn_wall_rs::{
    check_dir_exists,
    config::Args,
    listener_setup, print_schedule, sun_timings,
    time_track::Time,
    wallpaper_listener,
};
use std::{
    fs::canonicalize, process, sync::Arc,
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
    args = match Args::new(cli_args, cli_args_used) {
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
        Ok(s) => s,
    };

    //if day and night folders are being used, we need to only look at the second level of files
    //(files in the folders) which is what changing min_depth to 2 accomplishes
    if args.lat.is_some() && args.long.is_some() {
        min_depth = 2;
    }
    //println!("{:#?}", args);

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

