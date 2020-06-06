# dyn-wall-rs

![GitHub](https://img.shields.io/github/license/RAR27/dyn-wall-rs)
[![GitHub release (latest by date)](https://img.shields.io/github/v/release/RAR27/dyn-wall-rs)](https://github.com/RAR27/dyn-wall-rs)
[![Crates.io](https://img.shields.io/crates/v/dyn-wall-rs)](https://crates.io/crates/dyn-wall-rs)

A utility to allow you to set a dynamic wallpaper, and more.\
 Written in rust.

![demo][DEMO]

The images used in the gif above are from the collection [Lakeside by Louis Coyle](https://dynamicwallpaper.club/wallpaper/jculsb683ok).

## Introduction
The aim of dyn-wall-rs is to provide users with a very simple and easy way to implement a dynamic wallpaper, as well as a dynamic lockscreen, for their system. 


## Installation
You can download the binary from the [releases][RELEASES] page but if you prefer, you can install through one of the methods listed below
**NOTE: [Feh](https://feh.finalrewind.org/) needs to be installed for this to work**

### Cargo
First, install rust, and then run the following command:\
`cargo install dyn-wall-rs`

To update after installation, run:\
`cargo install dyn-wall-rs --force`

### Manual
  1. Download the latest binary from the [releases](RELEASES) page
  2. (**Optional**) To ensure the file you downloaded is correct and was not tampered with, do the following:
      1. Download the respective .sha256 file
      2. Run `sha256sum` on the .tar.gz file
      3. Compare the output of the command with the contents of the .sha256 file. If they are the same, then your file has not been tampered with
  3. Unpack the .tar.gz file by running\
`tar -zxvf dyn-wall-rs.tar.gz`
  4. You can now run it by running `./dyn-wall-rs` in the directory the binary was unpacked. It is recommended to place the binary in your $PATH, so you can use it from anywhere

## Usage
Firstly, create a directory and place all the wallpapers you want to cycle through within the directory. Make sure that they are named in numerical order ex. first wallpaper is named 1.png, second wallpaper is named 2.png, etc.

There are a few different ways to use dyn-wall-rs using the different flags, which are described in detail below
  * **-a, --auto \<DIRECTORY>**\
    Changes your wallpaper throughout the day in even increments.\
    For example if I have 12 wallpapers in my wallpaper directory, this option would change the wallpaper every 2 hours (24/12 = 2). Make sure the number of wallpapers in the directory can divide evenly into 1440 (number of minutes in a day). If it doesn't divide evenly into 1440, you may want to use the custom option.

  * **-c, --custom \<DIRECTORY>**\
    Changes your wallpaper based on custom times set through the config file located at ~/.config/dyn-wall-rs/config. When this is run for the first time, it will automatically create the config file with detailed instructions on how to set your own times for your wallpaper to change.

  * **-l, --lockscreen \<COMMAND>**\
    To have your lockscreen change as well, figure out the command that changes your lockscreen image. This command varies depending on your lockscreen. The command doesn't necessarily have to be a lockscreen command. You can use whatever command you want and have dyn-wall-rs send the wallpaper as an argument (ex. pywal). If the command includes arguments, wrap it in quotation marks.\
    ex. dyn-wall-rs -a /path/to/dir/ -l "betterlockscreen -u"

  * **-s, --schedule \<DIRECTORY>**\
    Prints out a schedule of the times at which the wallpaper would change if the auto option were to be used

Once you figure out which options you want to use and test it to make sure its working how you want it to, have the command autostart on boot.

## Planned Feature(s)
  * Ability to send multiple commands with the lockscreen argument, so you can do something like have dyn-wall-rs send the image path to pywall as well as betterlockscreen

[RELEASES]: https://github.com/RAR27/dyn-wall-rs/releases
[DEMO]: https://raw.githubusercontent.com/RAR27/dyn-wall-rs/master/demo.gif 
