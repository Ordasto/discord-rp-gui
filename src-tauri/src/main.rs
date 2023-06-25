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

static mut sender: Option<Sender<DiscordState>> = None;

struct DiscordState<'a> {
    discord_activity: Mutex<ActivityData<'a>>,
    handle: JoinHandle<()>,
}

// Implement serde::Serialize (maybe Deserialize) so i can pass it directly from javascript
// Although that might only apply to data passed to js, i wan't to pass data from js
#[derive(Clone)]
struct ActivityData<'a> {
    state_msg: &'a str,
    details: &'a str,
}

#[tauri::command]
fn update_status(stateMsg: &str, detailsMsg: &str, mut state: State<DiscordState>) -> String {
    // as i can "unpark" the thread from here, it might be better to park the thread instead of a loop and then when a change needs to be made, unpark it
    // make the change and then park again.
    let mut data = ActivityData {
        state_msg: stateMsg,
        details: detailsMsg,
    };
    state.handle.thread().unpark();


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
                    state_msg: "",
                    details: "",
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

    client.set_activity(activity).unwrap();
    
    // have a loop that parks the thread, then when it is externally unparked, modify activity, set_activity then park again
    
    loop {
        thread::park();
        let send_data = rx.recv().unwrap();
        // Do something with the data.

    }
}
