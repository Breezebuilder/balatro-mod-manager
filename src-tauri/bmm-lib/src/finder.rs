use log::error;
use log::info;
#[cfg(target_os = "windows")]
use std::fs::File;
#[cfg(target_os = "windows")]
use std::io::{BufReader, Read};
#[cfg(target_os = "windows")]
use std::path::Path;
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

use std::ffi::OsStr;
use sysinfo::System;

#[cfg(target_os = "windows")]
fn read_path_from_registry() -> Result<String, std::io::Error> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let steam_path = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Valve\\Steam")?;

    Ok(steam_path.get_value("InstallPath")?)
}

fn remove_unexisting_paths(paths: &mut Vec<PathBuf>) {
    let mut i = 0;
    while i < paths.len() {
        if !paths[i].exists() {
            paths.remove(i);
        } else {
            i += 1;
        }
    }
    info!("Found {} Balatro installations.", paths.len());
}

#[cfg(target_os = "windows")]
pub fn get_balatro_paths() -> Vec<PathBuf> {
    let steam_path = read_path_from_registry();
    let mut steam_path = steam_path.unwrap_or_else(|_| {
        error!("Could not read steam install path from Registry! Trying standard installation path in C:\\");
    });

    steam_path.push_str("\\steamapps\\libraryfolders.vdf");
    let libraryfolders_path = Path::new(&steam_path);
    if !libraryfolders_path.exists() {
        error!("'{}' not found.", libraryfolders_path.to_str().unwrap());
        return vec![];
    }

    let libraryfolders_file =
        File::open(libraryfolders_path).expect("Failed to open libraryfolders.vdf");
    let mut libraryfolders_contents = String::new();
    let mut libraryfolders_reader = BufReader::new(libraryfolders_file);
    libraryfolders_reader
        .read_to_string(&mut libraryfolders_contents)
        .expect("Failed to read libraryfolders.vdf");

    let mut paths: Vec<PathBuf> = vec![];
    let libraryfolders_contents = libraryfolders_contents.split("\n").collect::<Vec<&str>>();
    let mut libraryfolders_contents = libraryfolders_contents.iter();
    while let Some(line) = libraryfolders_contents.next() {
        if line.contains("\t\t\"path\"\t\t") {
            let path = line.split("\"").collect::<Vec<&str>>()[3];
            paths.push(PathBuf::from(path).join("steamapps\\common\\Balatro"));
        }
    }
    remove_unexisting_paths(&mut paths);
    pat
}

#[cfg(target_os = "macos")]
pub fn get_balatro_paths() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = vec![];
    match home::home_dir() {
        Some(path) => {
            let mut path = path;
            path.push("Library/Application Support/Steam/steamapps/common/Balatro");
            paths.push(path);
        }
        None => error!("Impossible to get your home dir!"),
    }
    remove_unexisting_paths(&mut paths);
    paths
}

#[cfg(target_os = "linux")]
pub fn get_balatro_paths() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = vec![];
    match home::home_dir() {
        Some(path) => {
            let mut path = path;
            path.push(".local/share/Steam/steamapps/common/Balatro");
            paths.push(path);
        }
        None => error!("Impossible to get your home dir!"),
    }
    remove_unexisting_paths(&mut paths);
    paths
}

pub fn is_steam_running() -> bool {
    let system = System::new_all();
    #[cfg(target_os = "windows")]
    let steam_processes = ["steam.exe", "steamservice.exe"];
    #[cfg(target_os = "macos")]
    let steam_processes = ["Steam", "steamservice"];
    #[cfg(target_os = "linux")]
    let steam_processes = ["steam", "steamservice"];

    steam_processes.iter().any(|&process_name| {
        system
            .processes_by_exact_name(OsStr::new(process_name))
            .next()
            .is_some()
    })
}
