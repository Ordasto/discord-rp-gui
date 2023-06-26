// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use discord_rich_presence::{
    activity::{self, Activity, Assets, Button, Timestamps, Party},
    DiscordIpc, DiscordIpcClient,
};
use serde::{Serialize, Deserialize};
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

#[derive(Clone, Serialize, Deserialize)]
struct Image{
    image_id:String,
    image_label:String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct ButtonData {
    url:String,
    label:String,
}

// Implement serde::Serialize (maybe Deserialize) so i can pass it directly from javascript
// Although that might only apply to data passed to js, i want to pass data from js
#[derive(Clone, Serialize, Deserialize)]
struct ActivityData {
    kill_app: bool,
    state: String,
    details: String,
    timestamps: Option<(i64, i64)>,
    party: Option<(String, i32)>,
    large_image: Image,
    small_image: Image,
    button_one: Option<ButtonData>, // Just using a tuple for now i cant be bothered
    button_two: Option<ButtonData>,
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
fn update_status(activity: ActivityData, mut state: State<DiscordState>) -> String {
    // let mut data = ActivityData::new();
    // data.state = stateMsg.to_string();
    // data.details = detailsMsg.to_string();
    // print!("{}",activity.);
    state.handle.thread().unpark();
    unsafe {
        SENDER.clone().unwrap().send(activity).unwrap();
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
        .state("are my speciality")
        .details("skeuomorphic bastardisations")
        .buttons(buttons)
        .assets(assets);

    client.set_activity(activity.clone()).unwrap();

    // have a loop that parks the thread, then when it is externally unparked, modify activity, set_activity then park again

    loop {
        thread::park();

        let sent_data: ActivityData = rx.recv().unwrap();

        if sent_data.kill_app {
            client.clear_activity().unwrap();
            client.close().unwrap();
            break;
        }

        let new_assets = Assets::new()
            .large_image(&sent_data.large_image.image_id)
            .large_text(&sent_data.large_image.image_label)
            .small_image(&sent_data.small_image.image_id)
            .small_text(&sent_data.small_image.image_label);

        let mut new_activity = Activity::new()
            .details(&sent_data.details)
            .state(&sent_data.state)
            .assets(new_assets)
            ;

        
        let button_one_local: ButtonData;
        let button_two_local: ButtonData;
    
        if sent_data.button_one.is_some() || sent_data.button_two.is_some() {
            let mut buttons: Vec<Button> = vec![];
            if let Some(btn) = sent_data.button_one {
                button_one_local = btn;
                buttons.push(
                    Button::new(
                        &button_one_local.label,
                        &button_one_local.url
                    )
                );
            }
            if let Some(btn) = sent_data.button_two {
                button_two_local = btn;
                buttons.push(
                    Button::new(
                        &button_two_local.label,
                        &button_two_local.url
                    )
                );
            }
            new_activity = new_activity.buttons(buttons);
        }
        


        client.set_activity(new_activity).unwrap();
    }
}
