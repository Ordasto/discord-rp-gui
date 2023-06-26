const { invoke } = window.__TAURI__.tauri;

let greetInputEl;
let updateMsg;

let state;
let details;
// might move the activity object outside and just send it whole in this func
async function updateStatus() {
	// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
	updateMsg.textContent = await invoke("update_status", { 
		activity: {
			kill_app: false,
			state: "are my speciality",
			details: "skeuomorphic bastardisations",
			timestamps: null,
			party: null,
			large_image: {
				image_id:"sadge2",
				image_label:"label21",
			},
			small_image: {
				image_id:"sadge2",
				image_label:":)",
			},
			// button_one:null,
			button_one: {
				url:"https://doc.rust-lang.org/rust-by-example/flow_control/if_let.html",
				label:"label",
			},
			button_two: {
				url:"https://motherfuckingwebsite.com",
				label:"test",
			},
		}
	});
}

window.addEventListener("DOMContentLoaded", () => {
	greetInputEl = document.querySelector("#greet-input");
	updateMsg = document.querySelector("#update-msg");
	state = document.querySelector("#state");
	details = document.querySelector("#details");
	document.querySelector("#update-form").addEventListener("submit", (e) => {
		e.preventDefault();
		updateStatus();
	});
});

