// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use discord_rich_presence::{
    activity::{self, Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
};
use std::{
    sync::Mutex,
    thread::{self, JoinHandle},
    time::Duration,
};
use tauri::State;
// mod discordrp;

struct Connection {
    connected: bool,
    handle: Option<JoinHandle<()>>,
}

struct DiscordState {
    connection: Mutex<Connection>,
    discord_activity: Mutex<ActivityData<'static>>,
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

    // *state.discord_activity.lock().unwrap() = data.clone();
    // let activity_mutex = state.discord_activity.lock().unwrap();

    let mut con = state.connection.lock().unwrap();

    if !con.connected {
        let handle = discord_init(&mut data); // Might be able to remove clone just cba right now

        con.handle = Some(handle);
        con.connected = true;

        return format!("New Connection");
    } else {
        // con.handle.as_ref().unwrap().thread().unpark();

        // if con.handle.as_ref().unwrap().is_finished() {
        //     con.connected = false;
        // }
        return format!("Already connected");
    }
}

fn main() {
    tauri::Builder::default()
        .manage(DiscordState {
            discord_activity: ActivityData {
                state_msg: "",
                details: "",
            }
            .into(),

            connection: Connection {
                connected: false,
                handle: None,
            }
            .into(),
        })
        .invoke_handler(tauri::generate_handler![update_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn discord_init(update_data: &mut ActivityData) -> thread::JoinHandle<()> {

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
        .state(&update_data.state_msg) // I know this is changed before being displayed, it was just to test the ability to change things after
        .details("just testing some stuff please")
        .buttons(buttons)
        .assets(assets);

    return thread::spawn(move || {
        client.set_activity(activity).unwrap();
        thread::park();
        // have a loop that parks the thread, then when it is externally unparked, modify activity, set_activity then park again

        // loop {

        // }
        client.clear_activity().unwrap();
    });
}
