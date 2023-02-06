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
use crate::{
    config::Args,
    errors::{ConfigFileErrors, Errors},
    time_track::Time,
};
use chrono::{Local, Timelike, Utc};
use clokwerk::{Scheduler, TimeUnits};
use dirs_next::data_dir;
use std::{env, error::Error, process, process::Command, sync::Arc, thread::sleep, time::Duration};
use std::{
    fs,
    fs::create_dir_all,
    fs::File,
    fs::OpenOptions,
    io::{Read, Write},
};
use walkdir::{DirEntry, WalkDir};

use run_script::ScriptOptions;
use unicase::UniCase;

#[cfg(not(windows))]
use std::env::consts::ARCH;

//crates used to change windows wallpaper
#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::{io, iter, os::raw::c_void, os::windows::ffi::OsStrExt};
#[cfg(windows)]
use winapi::um::winuser::{
    SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
};

pub mod config;
pub mod errors;
pub mod time_track;

const FULL_DAY: Time = Time {
    hours: 24,
    mins: 0,
    total_mins: 1440,
};
const MIDNIGHT: Time = Time {
    hours: 0,
    mins: 0,
    total_mins: 0,
};

pub fn wallpaper_current_time(
    dir: &str,
    progs: Arc<Option<Vec<String>>>,
    times: &[Time],
    backend: Arc<Option<String>>,
    min_depth: usize,
) -> Result<(), Box<dyn Error>> {
    let dir_iter = sorted_dir_iter(dir, min_depth);
    let dir_count = sorted_dir_iter(dir, min_depth);

    let dir_count: usize = dir_count.count();
    let mut commands_vec: Vec<Command> = vec![];
    let mut times_iter = times.iter();
    let curr_time = Time::new(Local::now().hour() * 60 + Local::now().minute());
    let loop_time = times_iter.next();
    let mut next_time = times_iter.next().unwrap_or(&FULL_DAY);
    let mut filepath_set: String = String::new();
    let mut last_image = String::new();
    let first_time = times[0];

    let mut loop_time = error_checking(times, loop_time, dir_count, None)?;

    //this loop is to find where the current time lays, and adjust the wallpaper based on that
    for file in dir_iter {
        //needed for the case where midnight is passed over in the middle of the stated times
        if loop_time > *next_time {
            loop_time = MIDNIGHT;
        }

        let filepath_temp = file.map_err(|_| Errors::FilePathError)?;
        let filepath_temp = filepath_temp.path();

        let last_image_temp = filepath_temp.to_str().unwrap();
        last_image = last_image_temp.to_owned();

        if curr_time >= loop_time && curr_time < *next_time {
            filepath_set.push_str(match filepath_temp.to_str() {
                Some(filepath) => Ok(filepath),
                None => Err(Errors::FilePathError),
            }?);

            //this is to send the file as an argument to the user specified program, if one was specified
            commands_vec_loader(&filepath_set, Arc::clone(&progs), &mut commands_vec);
        }
        loop_time = *next_time;
        next_time = times_iter.next().unwrap_or(&first_time);
    }

    //this is for the edge case where the current time is after the last time specified for the day, but before the first one specified for the day
    //in that case, the previous loop would push nothing to filepath_set, and so nothing would be sent to feh
    //what we want in this situation is for the file that is associated with the last time of the day to be sent as an argument to feh,
    //and to the user specified program
    if filepath_set.is_empty() {
        de_command_spawn(&last_image, backend)?;

        commands_vec_loader(&last_image, Arc::clone(&progs), &mut commands_vec);
        filepath_set = last_image;
    } else {
        de_command_spawn(&filepath_set, backend)?;
    }

    if let Some(progs) = progs.as_deref() {
        let mut prog_iter = progs.iter();
        for curr_command in commands_vec.iter_mut() {
            curr_command
                .spawn()
                .map_err(|_| Errors::ProgramRunError(String::from(prog_iter.next().unwrap())))?;
            println!(
                "The image {} has been sent as an argument to the specified program",
                filepath_set
            );
        }
    }
    Ok(())
}

pub fn wallpaper_listener(dir: String, args: Args, min_depth: usize) -> Result<(), Box<dyn Error>> {
    let mut scheduler = Scheduler::new();
    let mut sched_addto = scheduler.every(1.day()).at("0:00");
    let progs = Arc::new(args.programs);
    let backend = Arc::new(args.backend);
    let times = args.times.unwrap();
    let days = args.days;
    let days_val = days.unwrap_or(1);

    if days.is_none() {
        wallpaper_current_time(
            &dir,
            Arc::clone(&progs),
            &times,
            Arc::clone(&backend),
            min_depth,
        )?;

        for time in &times {
            let time_fmt = format!("{:02}:{:02}", time.hours, time.mins);
            sched_addto = sched_addto.and_every(1.day()).at(time_fmt.as_str());
        }

        let sched_closure = move || {
            let result = wallpaper_current_time(
                &dir,
                Arc::clone(&progs),
                &times,
                Arc::clone(&backend),
                min_depth,
            );

            match result {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            }
        };
        sched_addto.run(sched_closure);
    } else {
        sched_addto = sched_addto.and_every(days_val.day()).at("00:00");
        let curr_fp = file_data_load()?.into_iter().last().unwrap();
        set_wallpaper(&curr_fp, Arc::clone(&progs), Arc::clone(&backend))?;

        let sched_closure = move || {
            // append new chosen file name to the file
            // setting function will look at file name at bottom
            // and set accordingly.
            let filepath_set = update_wallpaper_days(&dir);
            let filepath_set = match filepath_set {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };
            set_wallpaper(&filepath_set, Arc::clone(&progs), Arc::clone(&backend)).unwrap();
        };
        sched_addto.run(sched_closure);
    }

    loop {
        scheduler.run_pending();
        sleep(Duration::from_millis(1000));
    }
}

fn commands_vec_loader(
    filepath_set: &str,
    progs: Arc<Option<Vec<String>>>,
    commands_vec: &mut Vec<Command>,
) {
    if let Some(prog_vec) = progs.as_deref() {
        for prog_str in prog_vec.iter() {
            let mut wall_sent = false;
            let mut prog_split = prog_str.split_whitespace();
            let mut curr_command = Command::new(prog_split.next().unwrap());
            for word in prog_split {
                //replacing !WALL with the filepath
                if word == "!WALL" {
                    curr_command.arg(filepath_set);
                    wall_sent = true;
                } else {
                    curr_command.arg(word);
                }
            }
            //if the filepath has been placed previously, this ensures that we dont place it again at the end
            if wall_sent == false {
                curr_command.arg(filepath_set);
            }
            commands_vec.push(curr_command);
        }
    }
}

pub fn auto_time_setup(dir: &str) -> (Result<Time, Errors>, Time) {
    let dir_count = WalkDir::new(dir).into_iter().count() - 1;
    let step_time = if dir_count == 0 {
        Err(Errors::NoFilesFoundError(dir.to_string()))
    } else {
        Ok(Time::new(((24.0 / dir_count as f32) * 60.0) as u32))
    };
    let loop_time = Time::default();

    (step_time, loop_time)
}

pub fn print_schedule(dir: &str, min_depth: usize, args: Args) -> Result<(), Box<dyn Error>> {
    let mut dir_iter = sorted_dir_iter(dir, min_depth);
    let dir_count = sorted_dir_iter(dir, min_depth).count();
    let mut sched_str: Vec<String> = vec![];
    let times = args.times.unwrap();
    let mut times_iter = times.iter();

    error_checking(&times, times_iter.next(), dir_count, args.days)?;

    for time in times_iter {
        let file = dir_iter
            .next()
            .ok_or(Errors::ConfigFileError(ConfigFileErrors::FileTimeMismatch))??;
        let file = file.file_name();
        sched_str.push(format!("Image: {:?} Time: {}", file, time.twelve_hour()));
    }

    for line in sched_str.iter() {
        println!("{}", line);
    }

    Ok(())
}

pub fn sorted_dir_iter(dir: &str, min_depth: usize) -> walkdir::IntoIter {
    WalkDir::new(dir)
        .sort_by(|a, b| {
            alphanumeric_sort::compare_str(
                a.path().to_str().expect("Sorting directory files failed"),
                b.path().to_str().expect("Sorting directory files failed"),
            )
        })
        .min_depth(min_depth)
        .into_iter()
}

fn error_checking(
    times: &[Time],
    loop_time: Option<&Time>,
    dir_count: usize,
    days: Option<u32>,
) -> Result<Time, Box<dyn Error>> {
    let times_iter_err = times.iter();
    let start_range = times
        .iter()
        .next()
        .ok_or(Errors::ConfigFileError(ConfigFileErrors::Empty))?;
    let mut start_range_other = times.iter().next().unwrap();
    let mut curr_range = start_range.to_owned();
    let mut curr_range_other = start_range.to_owned();
    let mut other_inited = false;
    let mut checked = vec![];

    //loop through and error check. When time passes midnight, another loop is required in order to
    //start error checking those timings properly, to avoid the false error of the previous time
    //being greater than the next time
    for time in times_iter_err {
        if *time > *start_range && *time > curr_range {
            curr_range = *time;
        } else if *time > *start_range && *time < curr_range {
            return Err(Errors::ConfigFileError(ConfigFileErrors::OutOfOrder).into());
        } else if *time < *start_range {
            if !other_inited {
                curr_range = FULL_DAY;
                start_range_other = time;
                curr_range_other = *time;
                other_inited = true
            } else if *time > *start_range_other && *time > curr_range_other {
                curr_range_other = *time;
            } else {
                return Err(Errors::ConfigFileError(ConfigFileErrors::OutOfOrder).into());
            }
        }
        if time.total_mins >= 24 * 60 {
            return Err(Errors::ConfigFileError(ConfigFileErrors::OutOfRange).into());
        }
        if checked.contains(time) {
            return Err(Errors::ConfigFileError(ConfigFileErrors::DuplicatesFound).into());
        }
        checked.push(*time);
    }
    if times.len() != dir_count && days.is_none() {
        return Err(Errors::ConfigFileError(ConfigFileErrors::FileTimeMismatch).into());
    }

    let loop_time = match loop_time {
        None => Err(Errors::ConfigFileError(ConfigFileErrors::Empty)),
        Some(time) => Ok(time),
    }?;
    Ok(*loop_time)
}

#[cfg(windows)]
fn de_command_spawn(
    filepath_set: &str,
    backend: Arc<Option<String>>,
) -> Result<(), Box<dyn Error>> {
    if backend.is_some() {
        eprintln!("NOTE: You are unable to select a backend on windows");
    }
    unsafe {
        let file = OsStr::new(filepath_set)
            .encode_wide()
            // append null byte
            .chain(iter::once(0))
            .collect::<Vec<u16>>();
        let successful = SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            file.as_ptr() as *mut c_void,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        ) == 1;

        if successful {
            println!("{} has been set as your wallpaper", filepath_set);
            Ok(())
        } else {
            Err(io::Error::last_os_error().into())
        }
    }
}

#[cfg(not(windows))]
fn de_command_spawn(
    filepath_set: &str,
    backend: Arc<Option<String>>,
) -> Result<(), Box<dyn Error>> {
    let backend = backend.as_deref();
    let gnome = vec![
        UniCase::new("gnome"),
        UniCase::new("gnome-xorg"),
        UniCase::new("ubuntu"),
        UniCase::new("deepin"),
        UniCase::new("pop"),
        UniCase::new("ubuntu:gnome"),
    ];
    let pantheon = UniCase::new("pantheon");
    let mate = UniCase::new("mate");
    let kde = vec![
        UniCase::new("plasma"),
        UniCase::new("neon"),
        UniCase::new("kde"),
        UniCase::new("/usr/share/xsessions/plasma"),
    ];
    let lxde = UniCase::new("lxde");
    let xfce = vec![
        UniCase::new("xfce"),
        UniCase::new("xubuntu"),
        UniCase::new("xfce session"),
    ];

    let curr_de = env::var("XDG_CURRENT_DESKTOP");
    let curr_de = match curr_de {
        Err(_) => String::from("Other"),
        Ok(de) => de,
    };
    let mut curr_de = UniCase::new(curr_de.as_str());

    let mut feh_handle = Command::new("feh");
    let feh_handle = feh_handle.arg("--bg-scale").arg(filepath_set);

    //Gnome, Ubuntu, Deepin, Pop
    let mut gnome_handle = Command::new("gsettings");
    let gnome_handle = gnome_handle
        .arg("set")
        .arg("org.gnome.desktop.background")
        .arg("picture-uri")
        .arg(format!("'file://{}'", filepath_set));

    //Pantheon
    let mut multiarch_dir = String::from("/usr/lib/");
    multiarch_dir.push_str(ARCH);
    multiarch_dir.push_str("-linux-gnu/");
    let mut pantheon_handle = Command::new(multiarch_dir + "io.elementary.contract.set-wallpaper");
    let pantheon_handle = pantheon_handle.arg(format!("{}", filepath_set));

    //kde
    let kde_script_beg = r#"
qdbus org.kde.plasmashell /PlasmaShell org.kde.PlasmaShell.evaluateScript "
    var allDesktops = desktops();
    print (allDesktops);
    for (i=0;i<allDesktops.length;i++) {
        d = allDesktops[i];
        d.wallpaperPlugin = 'org.kde.image';
        d.currentConfigGroup = Array('Wallpaper',
                                    'org.kde.image',
                                    'General');
        d.writeConfig('Image', 'file://"#;
    let kde_script_end = r#"')
        }""#;
    let kde_script = format!("{}{}{}", kde_script_beg, filepath_set, kde_script_end);

    //lxde
    let mut lxde_handle = Command::new("pcmanfm");
    let lxde_handle = lxde_handle.arg("--set-wallpaper").arg(filepath_set);

    //mate
    let mut mate_handle = Command::new("gsettings set org.mate.background picture-filename");
    let mate_handle = mate_handle
        .arg("set")
        .arg("org.mate.background")
        .arg("picture-uri")
        .arg(format!("'file://{}'", filepath_set));

    //let xfce_script_beg = r#"xfconf-query -c xfce4-desktop \
    //-p /backdrop/screen0/monitor0/workspace0/last-image \
    //-s ""#;
    let xfce_script_beg = "xfconf-query -c xfce4-desktop -l | grep last-image | while read path; do xfconf-query -c xfce4-desktop -p $path -s ";
    let xfce_script_end = r#"; done"#;
    let xfce_script = format!("{}{}{}", xfce_script_beg, filepath_set, xfce_script_end);

    //to avoid uninitialized variable error
    //safe because its not used unless custom backend specified
    let mut backend_split: Vec<&str> = vec![];

    let mut cust_backend = false;
    if let Some(back) = backend {
        curr_de = UniCase::new(back);
        cust_backend = true;
        for word in back.split_whitespace() {
            backend_split.push(word);
        }
    }

    if gnome.contains(&curr_de) {
        gnome_handle
            .spawn()
            .map_err(|_| Errors::ProgramRunError(String::from("Gnome Wallpaper Adjuster")))?;
    } else if lxde == curr_de {
        lxde_handle
            .spawn()
            .map_err(|_| Errors::ProgramRunError(String::from("LXDE Wallpaper Adjuster")))?;
    } else if pantheon == curr_de {
        pantheon_handle
            .spawn()
            .map_err(|_| Errors::ProgramRunError(String::from("Pantheon Wallpaper Adjuster")))?;
    } else if mate == curr_de {
        mate_handle
            .spawn()
            .map_err(|_| Errors::ProgramRunError(String::from("Mate Wallpaper Adjuster")))?;
    } else if kde.contains(&curr_de) {
        run_script::run(kde_script.as_str(), &vec![], &ScriptOptions::new())
            .map_err(|_| Errors::ProgramRunError(String::from("KDE Wallpaper Adjuster")))?;
    } else if xfce.contains(&curr_de) {
        run_script::run(xfce_script.as_str(), &vec![], &ScriptOptions::new())
            .map_err(|_| Errors::ProgramRunError(String::from("XFCE Wallpaper Adjuster")))?;
    } else if !cust_backend || curr_de == UniCase::new("feh") {
        feh_handle
            .spawn()
            .map_err(|_| Errors::ProgramRunError(String::from("Feh")))?;
    } else if cust_backend {
        let mut backend_split = backend_split.into_iter();
        let mut cust_handle = Command::new(backend_split.next().unwrap());
        let mut wall_sent = false;
        for word in backend_split {
            if word == "!WALL" {
                wall_sent = true;
                cust_handle.arg(filepath_set);
            } else {
                cust_handle.arg(word);
            }
        }

        if !wall_sent {
            cust_handle.arg(filepath_set);
        }

        cust_handle
            .spawn()
            .map_err(|_| Errors::ProgramRunError(curr_de.to_string()))?;
    } else {
        return Err(Errors::BackendNotFoundError(curr_de.to_string()).into());
    }

    println!("{} has been set as your wallpaper", filepath_set);
    Ok(())
}

pub fn sun_timings(
    dir: &str,
    lat: f64,
    long: f64,
    elevation: f64,
) -> Result<Vec<Time>, Box<dyn Error>> {
    let dir_night = format!("{}/night", dir);
    let dir_night = dir_night.as_str();
    let dir_day = format!("{}/day", dir);
    let dir_day = dir_day.as_str();
    let (dir_count_day, dir_count_night) = sun_timings_dir_counts(dir, dir_day, dir_night)?;
    if dir_count_day == 0 {
        return Err(Errors::NoFilesFoundError(String::from(dir_day)).into());
    } else if dir_count_night == 0 {
        return Err(Errors::NoFilesFoundError(String::from(dir_night)).into());
    }
    let mut times: Vec<Time> = vec![];
    let (sunrise, sunset) = sun_times::sun_times(Utc::today(), lat, long, elevation);
    let (sunset, sunrise) = (sunset.with_timezone(&Local), sunrise.with_timezone(&Local));
    let sunset = Time::new((sunset.hour() * 60) + sunset.minute());
    let sunrise = Time::new((sunrise.hour() * 60) + sunrise.minute());
    let step_time_day = Time::new((sunset.total_mins - sunrise.total_mins) / (dir_count_day));
    let step_time_night =
        Time::new((1440 - (sunset.total_mins - sunrise.total_mins)) / dir_count_night);
    let mut loop_time_night: Time;
    let mut loop_time_day = sunrise.to_owned();

    while loop_time_day <= sunset {
        if loop_time_day >= FULL_DAY {
            times.push(loop_time_day - FULL_DAY);
        } else {
            times.push(loop_time_day);
        }
        loop_time_day += step_time_day;
    }
    //poping off the last one (which will either be the sunset time, or be very close, and replace
    //it with the sunset time
    times.pop();
    loop_time_night = sunset.to_owned();

    while loop_time_night < (sunrise + FULL_DAY) {
        if loop_time_night >= FULL_DAY {
            times.push(loop_time_night - FULL_DAY);
        } else {
            times.push(loop_time_night);
        }
        loop_time_night += step_time_night;
    }

    //Because of the rounding that takes place when determining step time, it may throw off the
    //timing, possibly making it so that an extra time slips through
    //when this takes place, it would mean that the difference between the last time and the first
    //time (sunrise) is less than step_time_night, so in this case we would simply pop off the
    //extra time segment
    if i32::abs(sunrise.total_mins as i32 - times[times.len() - 1].total_mins as i32)
        < step_time_night.total_mins as i32
    {
        times.pop();
    }
    Ok(times)
}

fn sun_timings_dir_counts(
    dir: &str,
    dir_day: &str,
    dir_night: &str,
) -> Result<(u32, u32), Box<dyn Error>> {
    //checking if the directories exist
    if check_dir_exists(dir).is_err() {
        Err(Errors::FilePathError.into())
    } else if check_dir_exists(dir_night).is_err() || check_dir_exists(dir_day).is_err() {
        Err("Error: Make sure night and day directories are created within master directory".into())
    } else {
        //now we know directories exist, so lets get the counts of the night
        //and day directories and send it to sun_timings function to get vector
        //of times based on sunset and sunrise
        let dir_count_night = WalkDir::new(dir_night).min_depth(1).into_iter().count();
        let dir_count_day = WalkDir::new(dir_day).min_depth(1).into_iter().count();
        Ok((dir_count_day as u32, dir_count_night as u32))
    }
}
pub fn check_dir_exists(dir: &str) -> Result<(), Errors> {
    let mut dir_iter = WalkDir::new(dir).into_iter();

    if dir_iter.next().unwrap().is_err() {
        Err(Errors::DirNonExistantError(dir.to_string()))
    } else {
        Ok(())
    }
}

pub fn file_data_load() -> Result<Vec<String>, Box<dyn Error>> {
    let data_dir = data_dir().unwrap();
    create_dir_all(format!("{}/dyn-wall-rs", data_dir.to_str().unwrap()))?;
    let data_file = File::open(format!(
        "{}/dyn-wall-rs/visited",
        data_dir.to_str().unwrap()
    ));

    let mut data_file = data_file?;
    let mut contents: String = Default::default();
    data_file.read_to_string(&mut contents)?;
    contents = contents.trim().to_string();
    let splitted = contents.split("\n");
    let splitted = splitted.map(|x| x.to_string());
    let splitted: Vec<String> = splitted.collect();
    Ok(splitted)
}

pub fn file_data_save(contents: &str) -> Result<(), Box<dyn Error>> {
    let data_dir = data_dir().unwrap();
    let data_dir = data_dir.to_str().unwrap();
    println!("{}", data_dir);
    let filepath = format!("{}/dyn-wall-rs/visited", data_dir);
    let mut data_file = OpenOptions::new().write(true).append(true).open(filepath)?;
    let newlined = contents.to_string() + "\n";

    data_file.write_all(newlined.as_bytes())?;

    Ok(())
}

pub fn create_data_file() -> Result<bool, Box<dyn Error>> {
    let mut ret: bool = false;
    let data_dir = data_dir().unwrap();
    create_dir_all(format!("{}/dyn-wall-rs", data_dir.to_str().unwrap()))?;
    let mut data_file = File::open(format!(
        "{}/dyn-wall-rs/visited",
        data_dir.to_str().unwrap()
    ));
    if data_file.is_err() {
        ret = true;
        data_file = File::create(format!(
            "{}/dyn-wall-rs/visited",
            data_dir.to_str().unwrap()
        ));
        let mut data_file = data_file?;
        let contents = "";

        data_file.write_all(contents.as_bytes())?;
    }
    Ok(ret)
}

pub fn reset_file() -> Result<(), Box<dyn Error>> {
    let data_dir = data_dir().unwrap();
    let filepath = format!("{}/dyn-wall-rs/visited", data_dir.to_str().unwrap());
    fs::remove_file(filepath)?;
    create_data_file()?;
    Ok(())
}

pub fn set_wallpaper(
    filepath_set: &str,
    progs: Arc<Option<Vec<String>>>,
    backend: Arc<Option<String>>,
) -> Result<(), Box<dyn Error>> {
    let mut commands_vec: Vec<Command> = vec![];

    //this is to send the file as an argument to the user specified program, if one was specified
    commands_vec_loader(&filepath_set, Arc::clone(&progs), &mut commands_vec);

    //this is for the edge case where the current time is after the last time specified for the day, but before the first one specified for the day
    //in that case, the previous loop would push nothing to filepath_set, and so nothing would be sent to feh
    //what we want in this situation is for the file that is associated with the last time of the day to be sent as an argument to feh,
    //and to the user specified program
    de_command_spawn(&filepath_set, backend)?;

    if let Some(progs) = progs.as_deref() {
        let mut prog_iter = progs.iter();
        for curr_command in commands_vec.iter_mut() {
            curr_command
                .spawn()
                .map_err(|_| Errors::ProgramRunError(String::from(prog_iter.next().unwrap())))?;
            println!(
                "The image {} has been sent as an argument to the specified program",
                filepath_set
            );
        }
    }
    Ok(())
}

pub fn update_wallpaper_days(dir: &str) -> Result<String, Box<dyn Error>> {
    let dir_iter = sorted_dir_iter(dir, 1);

    let mut filepath_set: String = String::new();
    let old = file_data_load()?;

    for file in dir_iter {
        let filepath = file?;
        let filepath = filepath.path().to_str().unwrap();
        let filepath = filepath.to_string();
        if !old.contains(&filepath) {
            filepath_set.push_str(&filepath);
            break;
        }
    }

    // if we didn't encounter file that hasn't been visited,
    // this means all have been visited, and so we need to reset
    if filepath_set.is_empty() {
        reset_file()?;
        let temp = sorted_dir_iter(dir, 1).next().unwrap();
        filepath_set.push_str(temp?.path().to_str().unwrap());
    }
    file_data_save(&filepath_set)?;
    Ok(filepath_set)
}
