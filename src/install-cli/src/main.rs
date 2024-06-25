use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::{exit, Command},
};

//Imports only used in windows
#[cfg(target_os = "windows")]
use std::io::stdin;

use mslnk::ShellLink;
use rust_embed::Embed;
use whiskers_launcher_rs::paths::{
    get_app_dir, get_app_resources_dir, get_app_resources_icons_dir,
};

//Imports only used in linux
#[cfg(target_os = "linux")]
use {
    fs_extra::dir::CopyOptions,
    std::{fs, process::Command},
    whiskers_launcher_rs::paths::get_app_resources_dir,
};

#[cfg(target_os = "windows")]
fn press_to_close() {
    let mut s = String::new();
    println!("\nPress enter to close");
    stdin().read_line(&mut s).unwrap();
    exit(0);
}

pub fn is_wayland() -> bool {
    match env::var("XDG_SESSION_TYPE") {
        Ok(session) => &session.to_lowercase() == "wayland",
        Err(_) => false,
    }
}

pub fn write_file(path: PathBuf, bytes: &[u8]) {
    let mut file = File::create(&path).expect("Error creating file");
    file.write_all(bytes).expect("Error writing file");
}

#[cfg(target_os = "windows")]
#[derive(Embed)]
#[folder = "files/windows/binaries/"]
struct WindowsBinaries;

#[cfg(target_os = "windows")]
#[derive(Embed)]
#[folder = "files/windows/scripts/"]
struct WindowsScripts;

#[derive(Embed)]
#[folder = "files/general/icons/"]
struct Icons;

fn main() {
    let binary_path = env::current_exe().expect("Error getting path");
    let binary_dir = binary_path.parent().unwrap();

    let mut installation_files_dir = binary_dir.to_owned();
    installation_files_dir.push("installation-files");

    let mut logo = installation_files_dir.to_owned();
    logo.push("whiskers-launcher.png");

    let mut icons_dir = installation_files_dir.to_owned();
    icons_dir.push("resources");
    icons_dir.push("icons");

    #[cfg(target_os = "linux")]
    if env::consts::OS == "linux" {
        let resources_dir = get_app_resources_dir();

        let mut launcher_bin = installation_files_dir.to_owned();
        launcher_bin.push("whiskers-launcher");

        let mut companion_bin = installation_files_dir.to_owned();
        companion_bin.push("whiskers-launcher-companion");

        let mut desktop_file = installation_files_dir.to_owned();
        desktop_file.push("whiskers-launcher.desktop");

        let copy_binaries_cmd = format!(
            "sudo cp '{}' '{}' /usr/bin",
            launcher_bin.into_os_string().into_string().unwrap(),
            companion_bin.into_os_string().into_string().unwrap()
        );

        let copy_binaries_result = Command::new("sh")
            .arg("-c")
            .arg(copy_binaries_cmd)
            .output()
            .expect("Error copying binaries");

        if !copy_binaries_result.status.success() {
            eprintln!(
                "Error while copying files: {}",
                String::from_utf8(copy_binaries_result.stderr).unwrap()
            );

            exit(1);
        }

        let copy_logo_cmd = format!(
            "sudo cp '{}' /usr/share/pixmaps",
            logo.into_os_string().into_string().unwrap()
        );

        let copy_logo_result = Command::new("sh")
            .arg("-c")
            .arg(copy_logo_cmd)
            .output()
            .expect("Error copying logo");

        if !copy_logo_result.status.success() {
            eprintln!(
                "Error copying logo: {}",
                String::from_utf8(copy_logo_result.stderr).unwrap()
            );

            exit(1);
        }

        let install_desktop_cmd = format!(
            "sudo desktop-file-install -m 644 --dir /usr/share/applications {}",
            desktop_file.into_os_string().into_string().unwrap()
        );

        let install_desktop_result = Command::new("sh")
            .arg("-c")
            .arg(install_desktop_cmd)
            .output()
            .expect("Error installing desktop file");

        if !install_desktop_result.status.success() {
            eprintln!(
                "Error installing desktop file: {}",
                String::from_utf8(install_desktop_result.stderr).unwrap()
            );

            exit(1);
        }

        if !&resources_dir.exists() {
            fs::create_dir_all(&resources_dir).expect("❌ Error creating resources directory");
        }

        fs_extra::dir::copy(
            &icons_dir,
            &resources_dir,
            &CopyOptions::new().overwrite(true).to_owned(),
        )
        .expect("Error copying app icons");

        match is_wayland() {
            true => println!("Note: Wayland was detected. You need to manually make a shortcut for the app on your DE/WM. You can use 'whiskers-launcher' or 'WEBKIT_DISABLE_COMPOSITING_MODE=1 whiskers-launcher' if you problems with the app not launching"),
            false => println!("Note: The default launch shortcut is 'ctrl + space'"),
        }

        println!("✅ Installed");
    }

    #[cfg(target_os = "windows")]
    if env::consts::OS == "windows" {
        let app_dir = get_app_dir();
        let resources_dir = get_app_resources_dir();

        let mut scripts_dir = resources_dir.to_owned();
        scripts_dir.push("scripts");

        let icons_dir = get_app_resources_icons_dir();

        if !app_dir.exists() {
            fs::create_dir_all(&app_dir).expect("Error creating app dir");
        }

        if !resources_dir.exists() {
            fs::create_dir_all(&resources_dir).expect("Error creating resources dir");
        }

        if !scripts_dir.exists() {
            fs::create_dir_all(&scripts_dir).expect("Error creating scripts dir");
        }

        if !icons_dir.exists() {
            fs::create_dir_all(&icons_dir).expect("Error creating icons dir");
        }

        for file in WindowsBinaries::iter() {
            if let Some(content) = WindowsBinaries::get(&file) {
                let mut path = app_dir.to_owned();
                path.push(file.to_string());

                write_file(path, content.data.as_ref());
            }
        }

        for file in WindowsScripts::iter() {
            if let Some(content) = WindowsScripts::get(&file) {
                let mut path = scripts_dir.to_owned();
                path.push(file.to_string());

                write_file(path, content.data.as_ref());
            }
        }

        for file in Icons::iter() {
            if let Some(content) = Icons::get(&file) {
                let mut path = icons_dir.to_owned();
                path.push(file.to_string());

                write_file(path, content.data.as_ref());
            }
        }

        // Create the shortcut
        let mut shortcut_path =
            Path::new(&env::var("APPDATA").expect("Error getting environment variable")).to_owned();

        shortcut_path.push("Microsoft\\Windows\\Start Menu\\Programs\\Whiskers-Launcher.lnk");

        let mut target_path = app_dir.to_owned();
        target_path.push("whiskers-launcher-companion.exe");

        let link = ShellLink::new(target_path.into_os_string().into_string().unwrap())
            .expect("Error initializing link");

        link.create_lnk(shortcut_path.into_os_string().into_string().unwrap())
            .expect("Error creating link");

        let mut companion_path = app_dir.to_owned();
        companion_path.push("whiskers-launcher-companion.exe");

        Command::new("cmd")
            .arg("/c")
            .arg(companion_path.into_os_string().into_string().unwrap())
            .spawn()
            .expect("Error opening companion app");

        println!("✅ Installed");

        press_to_close();
    }
}
