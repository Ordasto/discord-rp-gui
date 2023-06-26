// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use discord_rich_presence::{
    activity::{self, Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
};
use std::{
    sync::{Mutex, mpsc::{channel, Receiver, Sender}, self},
    thread::{self, JoinHandle},
    time::Duration, fmt::format,
};
use tauri::State;

static mut sender: Option<Sender<ActivityData>> = None;

struct DiscordState {
    discord_activity: Mutex<ActivityData>,
    handle: JoinHandle<()>,
}

// Implement serde::Serialize (maybe Deserialize) so i can pass it directly from javascript
// Although that might only apply to data passed to js, i wan't to pass data from js
#[derive(Clone)]
struct ActivityData {
    state_msg: String,
    details: String,
}

#[tauri::command]
fn update_status(stateMsg: &str, detailsMsg: &str, mut state: State<DiscordState>) -> String {
    let mut data = ActivityData {
        state_msg: stateMsg.to_string(),
        details: detailsMsg.to_string(),
    };
    state.handle.thread().unpark();
    unsafe{
        sender.clone().unwrap().send(data).unwrap();
    }

    return format!("no fucking idea");
}

fn main() {

    let (tx,rx) = channel();

    unsafe {
        sender = Some(tx);
    }

    let discord_handle = thread::spawn(|| {discord_init(rx)});
    tauri::Builder::default()
        .manage(
            DiscordState {
                discord_activity: ActivityData {
                    state_msg: String::new(),
                    details: String::new(),
                }
                .into(),
                handle: discord_handle,
        })
        .invoke_handler(tauri::generate_handler![update_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn discord_init(rx: Receiver<ActivityData>){

    let mut client = DiscordIpcClient::new("1122014998231781407").unwrap();

    client.connect().unwrap();

    let mut buttons: Vec<Button> = vec![];
    let button: Button = Button::new("Goof", "https://www.youtube.com/watch?v=vz0Aqlm22sk");
    let button2: Button = Button::new(
        "Dun goofed em",
        "https://www.youtube.com/watch?v=kdKIBYkEQv0",
    );

    buttons.push(button);
    buttons.push(button2);

    let assets: Assets = Assets::new()
        .large_image("big_goofy_face")
        .large_text("HUH?")
        .small_image("sadge2")
        .small_text("Huh?");

    let activity = activity::Activity::new()
        .state("Multi threading will be the end of me") // I know this is changed before being displayed, it was just to test the ability to change things after
        .details("just testing some stuff please")
        .buttons(buttons)
        .assets(assets);

    client.set_activity(activity.clone()).unwrap();
    
    // have a loop that parks the thread, then when it is externally unparked, modify activity, set_activity then park again
    
    loop {
        thread::park();
        let send_data = rx.recv().unwrap();

        let new_activity = activity.clone()
        .details(&send_data.details)
        .state(&send_data.state_msg)
        ;


        client.set_activity(new_activity).unwrap();
        // Do something with the data.

    }
}
