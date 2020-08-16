use crate::Time;
use serde::{Deserialize, Serialize};
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
        help = "Sets the wallpaper based on the current time and changes the wallpaper throughout the day. The wallpaper will change based on the user specified times within the config file or, if custom timings are not set, it will automatically divide the wallpapers into equal parts throughout the day.",
        conflicts_with = "Schedule"
    )]
    pub directory: Option<String>,

    #[structopt(
        short = "p",
        long = "programs",
        value_name = "COMMAND",
        help = r#"Sends image as argument to command specified. Use alongside listener or custom. If the command itself contains arguments, wrap in quotation ex. dyn-wall-rs -a /path/to/dir -l "betterlockscreen -u""#
    )]
    pub programs: Option<Vec<String>>,

    #[structopt(
        short,
        long,
        value_name = "DIRECTORY",
        help = "Will present you with a schedule of when your wallpaper will change",
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
