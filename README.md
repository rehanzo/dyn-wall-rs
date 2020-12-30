# dyn-wall-rs

![GitHub](https://img.shields.io/github/license/RAR27/dyn-wall-rs)
[![GitHub release (latest by date)](https://img.shields.io/github/v/release/RAR27/dyn-wall-rs)](https://github.com/RAR27/dyn-wall-rs)
[![Crates.io](https://img.shields.io/crates/v/dyn-wall-rs)](https://crates.io/crates/dyn-wall-rs)
[![AUR](https://img.shields.io/aur/version/dyn-wall-rs)](https://aur.archlinux.org/packages/dyn-wall-rs/)

A utility to allow you to set a dynamic wallpaper and more.\
 Written in rust.

![demo][DEMO]

The images used in the gif above are from the collection [Lakeside by Louis Coyle](https://dynamicwallpaper.club/wallpaper/jculsb683ok).

## Introduction
The aim of dyn-wall-rs is to provide users with a very simple and easy way to implement a dynamic wallpaper, as well as the setup of related things, such as the implementation of a dynamic lockscreen.


## Installation
You can download the binary from the [releases][RELEASES] page but if you prefer, you can install through one of the methods listed below.\
**NOTE: [Feh](https://feh.finalrewind.org/) needs to be installed if you are using a Window Manager**

### AUR
For those using Arch Linux you can find the package on the AUR [here](https://aur.archlinux.org/packages/dyn-wall-rs/). However, if you're using an AUR helper, the package can be installed through that. For example, If using [yay](https://github.com/Jguer/yay), run the following command:
```
yay -S dyn-wall-rs
```
**Looking for maintainer for the AUR package. Email me at rehanalirana@tuta.io if you are interested.**

### Cargo
First, install rust, and then run the following command:
```
cargo install dyn-wall-rs
```
To update after installation, run:
```
cargo install dyn-wall-rs --force
```

### Manual
#### Unix
  1. Download the latest binary from the [releases](RELEASES) page
  2. (**Optional**) To ensure the file you downloaded is correct and was not tampered with, do the following:
      1. Download the respective `.sha256` file
      2. Run `sha256sum` on the `.tar.gz` file
      3. Compare the output of the command with the contents of the `.sha256` file. If they are the same, then your file has not been tampered with
  3. Unpack the `.tar.gz file` by running\
`tar -zxvf dyn-wall-rs.tar.gz`
  4. You can now run it by running `./dyn-wall-rs` in the directory the binary was unpacked. It is recommended to place the binary in your $PATH (ex. `/usr/bin`, which is commonly used), so you can use it from anywhere

#### Windows
  1. Download the latest binary from the [releases](RELEASES) page
  2. (**Optional**) To ensure the file you downloaded is correct and was not tampered with, do the following:
      1. Download the respective `.sha256` file
      2. Open PowerShell, move to the directory contining the zip, and run\
      `Get-FileHash dyn-wall-rs-windows.zip -Algorithm SHA256 | Format-List`
      3. Compare the sha256 the command provides with the contents of the `.sha256` file. If they are the same, then your file has not been tampered with
  3. Unzip the `.zip` file
  4. You can now run it by opening up PowerShell and running `./dyn-wall-rs` in the directory the binary was unpacked. It is recommended to place the binary in your $PATH, so you can use it from anywhere

## Usage
Firstly, create a directory and place all the wallpapers you want to cycle through within the directory. Make sure that they are named in numerical order ex. first wallpaper is named 1.png, second wallpaper is named 2.png, etc.

### Command Line
There are a few different ways to use dyn-wall-rs from the command line using the different flags, which are described in detail below
  * **-d, --directory \<DIRECTORY>**\
    Changes your wallpaper throughout the day with the images in the directory. If custom timings are not specified through the config file, it changes in even increments throughout the day.\
    For example, if I have 12 wallpapers in my wallpaper directory, this option would change the wallpaper every 2 hours (24/12 = 2). Make sure the number of wallpapers in the directory can divide evenly into 1440 (number of minutes in a day). If it doesn't divide evenly into 1440, you may want to place custom timings in the configuration file.\
    If timings are specified through the configuration file, then the wallpapers will change based on those timings. More information on custom timings can be found within the automatically created config file.

  * **-p, --program \<COMMAND>**\
    Will send the wallpaper as an argument to the specified program when the wallpaper is set to change. Using this feature, you can have your lockscreen change alongside your wallpaper. If the command includes arguments, wrap it in quotation marks.\
    ex. `dyn-wall-rs -d /path/to/dir/ -p "betterlockscreen -u"`
    
    To be able to send arguments *after* the wallpaper argument, use `!WALL` to specify where the wallpaper argument is to be placed, and add the rest of the arguments. `!WALL` will be explanded to the path of the wallpaper to be set at the current time.\
    ex. `dyn-wall-rs -d /path/to/dir -p "betterlockscreen -u !WALL -b 1"`
    
    You are also able to specifiy multiple programs to be synced with the wallpaper. Simply just insert the program names one after the other
    ex. `dyn-wall-rs -d /path/to/dir -p "betterlockscreen -u" "echo"

  * **-s, --schedule**\
    Prints out a schedule of the times at which the wallpaper will change depending on your settings. Use alongside the `--directory` option.\
    **Note: Cannot be set through config file.**
    
  * **-b, --backend \<BACKEND>**\
    Uses the specified method as the backend to change the wallpaper. Type a supported DE name to use that DE's wallpaper changing command (Case insensitive), or type out a custom command to use as a backend. Similar to the `program` option, you can use `!WALL` in place of where the path of the wallpaper should be.
    
  * **--lat \<LATITUDE>**\
    Latitude of current location. Requires the use of the `long` option as well.
    
  * **--long \<LONGITUDE>**\
    Longitude of current location. Requires the use of the `lat` option as well.
    
  * **--elevation \<ELEVATION>**\
    Elevation of current location. Optional. Use alongside `long` and `lat` options for a more accurate sunset and sunrise reading. Expressed in meters above sea level.

Once you figure out which options you want to use and test it to make sure its working how you want it to, have the command autostart on boot.

### Config File
dyn-wall-rs can also be configured through a config file. When you run the program for the first time, a config file will be created at `~/.config/dyn-wall-rs/config.toml` for Unix systems, and `C:\Users\<USER NAME>\AppData\Roaming\dyn-wall-rs.toml` on Windows. 

Through this config file, you can use the same configuration options as through the command line (except the `schedule` option), as well as use your own custom timings. If you would like to configure certain parameters from the config file, and others from the command line, you are able to do so. More details can be found in the automatically created config file.

### Syncing to the sun
In order to sync the changing of wallpapers according to the sunset and sunrise timings, create directories within the master directory named `night` and `day`. This will cycle through the wallpapers in the `day` directory if the current time is before the sunset time, and will cycle through the wallpapers in the `night` directory. After the directories are created and the wallpapers are placed in them, specify your latitude, longitude, and elevation (optional), and let the program do its work! You can find your coordinates through [this](https://www.mapcoordinates.net/en) website.

## Supported Desktop Environments
  * Windows
  * Gnome
  * Ubuntu
  * Pantheon
  * Deepin
  * Pop
  * KDE
  * LXDE
  * XFCE
  * Window Managers that can have their wallpaper set using Feh

[RELEASES]: https://github.com/RAR27/dyn-wall-rs/releases
[DEMO]: https://raw.githubusercontent.com/RAR27/dyn-wall-rs/master/demo.gif 
