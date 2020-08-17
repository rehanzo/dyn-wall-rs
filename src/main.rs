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
use crate::errors::{ConfigFileErrors, Errors};
use clap::AppSettings;
use dyn_wall_rs::{
    auto_time_setup, check_dir_exists, config::Args, print_schedule, sun_timings, time_track::Time,
    wallpaper_listener,
};
use std::{fs::canonicalize, sync::Arc};
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
    //min depth of what files should be looked at, will remain as 1 if not syncing with sun, will
    //change to 2 if syncing with sun to ignore the directory names, focusing just on the files
    let mut min_depth = 1;

    match Args::mixed(cli_args, cli_args_used) {
        Err(e) => {
            eprintln!("{}", e);
        }
        Ok(mut args) => {
            //if day and night folders are being used, we need to only look at the second level of files
            //(files in the folders) which is what changing min_depth to 2 accomplishes
            if args.lat.is_some() && args.long.is_some() {
                min_depth = 2;
            }

            if let Some(dir) = &args.directory {
                let dir = dir.as_str();
                let dir_count = WalkDir::new(dir).min_depth(min_depth).into_iter().count();
                let dir = canonicalize(dir).expect("Failed to canonicalize");
                let dir = dir.to_str().expect("Couldn't convert to string");

                if args.times.is_none() {
                    if 1440 % dir_count != 0 || dir_count == 0 {
                        eprintln!("{}", Errors::CountCompatError(dir_count));
                    } else {
                        let (step_time, mut loop_time) = auto_time_setup(dir);
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
            }
        }
    }
}
