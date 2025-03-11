use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

use coreaudio_sys::*;
use std::mem;
use std::ptr;

#[derive(Default)]
struct AppState {
    is_muted: bool,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Function to get the default input device ID
fn get_default_input_device() -> Option<AudioObjectID> {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDefaultInputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };

    let mut device_id: AudioObjectID = 0;
    let size = mem::size_of::<AudioObjectID>() as u32;

    let status = unsafe {
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &property_address,
            0,
            ptr::null(),
            &mut (size as u32),
            &mut device_id as *mut _ as *mut _,
        )
    };

    if status == 0 && device_id != 0 {
        Some(device_id)
    } else {
        None
    }
}

fn get_mute_state(
    device_id: u32,
    size: usize,
    property_address: AudioObjectPropertyAddress,
) -> (i32, u32) {
    let mut mute_state: u32 = 0;

    let status = unsafe {
        AudioObjectGetPropertyData(
            device_id,
            &property_address,
            0,
            ptr::null(),
            &mut (size as u32),
            &mut mute_state as *mut _ as *mut _,
        )
    };

    return (status, mute_state);
}

#[tauri::command]
fn toggle_mic() {
    if let Some(device_id) = get_default_input_device() {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyMute,
            mScope: kAudioDevicePropertyScopeInput, // Input scope for mic
            mElement: kAudioObjectPropertyElementMaster,
        };

        let size = mem::size_of::<u32>();
        let (get_mute_state_status, mute_state) = get_mute_state(device_id, size, property_address);

        if get_mute_state_status == 0 {
            let new_state = if mute_state == 0 { 1 } else { 0 };

            let status = unsafe {
                AudioObjectSetPropertyData(
                    device_id,
                    &property_address,
                    0,
                    ptr::null(),
                    size as u32,
                    &new_state as *const _ as *const _,
                )
            };

            if status == 0 {
                println!(
                    "Microphone {}",
                    if new_state == 1 { "Muted" } else { "Unmuted" }
                );
            } else {
                eprintln!("Failed to set microphone mute state");
            }
        } else {
            eprintln!("Failed to get microphone mute status");
        }
    } else {
        eprintln!("No input device found!");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let toggle_i =
                MenuItem::with_id(app, "toggle_mute", "Toggle mute", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_i, &quit_i])?;
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        println!("quit menu item was clicked");
                        app.exit(0);
                    }
                    "toggle_mute" => {
                        println!("Toggle mute menu item was clicked");
                        toggle_mic();
                    }
                    _ => {
                        println!("menu item {:?} not handled", event.id);
                    }
                })
                .build(app)?;
            Ok(())
        })
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, toggle_mic])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
