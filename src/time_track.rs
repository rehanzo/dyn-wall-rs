/*
    dyn-wall-rs 1.0
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
use crate::ConfigFileErrors;
use crate::Errors;
use std::{
    ops::{Add, AddAssign, Sub, SubAssign},
    str::FromStr,
};

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct Time {
    pub total_mins: u32,
    pub hours: u32,
    pub mins: u32,
}

impl Time {
    pub fn new(total_mins: u32) -> Self {
        let mut hours = 0;
        let mut mins = total_mins;

        while mins >= 60 {
            hours += 1;
            mins -= 60;
        }

        Time {
            total_mins,
            hours,
            mins,
        }
    }

    pub fn twelve_hour(&self) -> String {
        match self.hours {
            0 => format!("12:{:02} a.m.", self.mins),
            1..=11 => format!("{}:{:02} a.m.", self.hours, self.mins),
            12 => format!("12:{:02} p.m.", self.mins),
            _ => format!("{}:{:02} p.m.", (self.hours - 12), self.mins),
        }
    }
}

impl FromStr for Time {
    type Err = Errors;

    fn from_str(time_str: &str) -> Result<Self, Self::Err> {
        let mut time_split = time_str.split(':');
        let hours = time_split
            .next()
            .ok_or_else(|| Errors::ConfigFileError(ConfigFileErrors::FormattingError))?
            .parse::<u32>()
            .map_err(|_| Errors::ConfigFileError(ConfigFileErrors::FormattingError))?;
        let mins = time_split
            .next()
            .ok_or_else(|| Errors::ConfigFileError(ConfigFileErrors::FormattingError))?
            .parse::<u32>()
            .map_err(|_| Errors::ConfigFileError(ConfigFileErrors::FormattingError))?;

        if mins >= 60 && hours >= 24 {
            Err(Errors::ConfigFileError(ConfigFileErrors::FormattingError))
        } else {
            let total_mins = hours * 60 + mins;

            Ok(Time {
                hours,
                mins,
                total_mins,
            })
        }
    }
}

impl Add for Time {
    type Output = Time;
    fn add(self, other: Time) -> Time {
        Time::new(self.total_mins + other.total_mins)
    }
}

impl AddAssign for Time {
    fn add_assign(&mut self, other: Self) {
        *self = Time::new(self.total_mins + other.total_mins);
    }
}

impl Sub<Time> for Time {
    type Output = Time;

    fn sub(self, other: Time) -> Time {
        Time::new(self.total_mins - other.total_mins)
    }
}

impl Sub<u32> for Time {
    type Output = Time;

    fn sub(self, other: u32) -> Time {
        Time::new(self.total_mins - other)
    }
}

impl SubAssign for Time {
    fn sub_assign(&mut self, other: Self) {
        *self = Time::new(self.total_mins - other.total_mins);
    }
}

impl Default for Time {
    fn default() -> Time {
        Time {
            total_mins: 0,
            hours: 0,
            mins: 0,
        }
    }
}
