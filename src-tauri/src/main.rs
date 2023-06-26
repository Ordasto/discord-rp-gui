// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use discord_rich_presence::{
    activity::{self, Activity, Assets, Button, Timestamps, Party},
    DiscordIpc, DiscordIpcClient,
};
use serde::Serialize;
use std::{
    fmt::format,
    sync::{
        self,
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tauri::State;



static mut SENDER: Option<Sender<ActivityData>> = None;

struct DiscordState {
    discord_activity: Mutex<ActivityData>,
    handle: JoinHandle<()>,
}

#[derive(Clone,Serialize)]
struct Image{
    image_id:String,
    image_label:String,
}

// Implement serde::Serialize (maybe Deserialize) so i can pass it directly from javascript
// Although that might only apply to data passed to js, i want to pass data from js
#[derive(Clone, Serialize)]
struct ActivityData {
    kill_app: bool,
    state: String,
    details: String,
    timestamps: Option<Timestamps>,
    party: Option<(String, i32)>,
    large_image: Image,
    small_image: Image,
    button_one: Option<(String, String)>, // Just using a tuple for now i cant be bothered
    button_two: Option<(String, String)>,
    // not doing secrets right now, cba
    // secrets: 

}

impl ActivityData {
    fn new() -> Self {
        // defualt empty string of two spaces
        // discord doesn't accept activities with empty or 1 char strings
        let def_str = "  ".to_string();
        return ActivityData {
            kill_app: false,
            state: def_str.clone(),
            details: def_str.clone(),
            timestamps: None,
            party: None,
            large_image: Image{image_id: def_str.clone(), image_label: def_str.clone()},
            small_image: Image{image_id: def_str.clone(), image_label: def_str.clone()},
            button_one: None,
            button_two: None,
        };
    }
}


#[tauri::command]
fn update_status(stateMsg: &str, detailsMsg: &str, mut state: State<DiscordState>) -> String {
    let mut data = ActivityData::new();
    data.state = stateMsg.to_string();
    data.details = detailsMsg.to_string();

    state.handle.thread().unpark();
    unsafe {
        SENDER.clone().unwrap().send(data).unwrap();
    }

    return format!("no fucking idea");
}

fn main() {
    let (tx, rx) = channel();

    unsafe {
        SENDER = Some(tx);
    }

    let discord_handle = thread::spawn(|| discord_init(rx));
    tauri::Builder::default()
        .manage(DiscordState {
            discord_activity: ActivityData::new().into(),
            handle: discord_handle,
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                // TELL DISCORD RP TO CLOSE CONNECTIION

                // this doesn't work
                // handle.unpark();
                // unsafe {
                //     SENDER.clone().unwrap().send(ActivityData {
                //         kill_app: true,
                //         state_msg: "".to_string(),
                //         details: "".to_string(),
                //     }).unwrap();
                // }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![update_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn discord_init(rx: Receiver<ActivityData>) {
    let mut client = DiscordIpcClient::new("1122968332698656870").unwrap();

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
        .state("Multi threading will be the end of me")
        .details("just testing some stuff please")
        .buttons(buttons)
        .assets(assets);

    client.set_activity(activity.clone()).unwrap();

    // have a loop that parks the thread, then when it is externally unparked, modify activity, set_activity then park again

    loop {
        thread::park();

        let send_data = rx.recv().unwrap();

        if send_data.kill_app {
            client.clear_activity().unwrap();
            client.close().unwrap();
            break;
        }

        let assets = Assets::new().large_image("sadge2").large_text("test");

        let new_activity = activity
            .clone()
            .details(&send_data.details)
            .state(&send_data.state)
            .assets(assets);

        // let new_activity = new_activity.buttons(vec![]);

        client.set_activity(new_activity).unwrap();
    }
}
