const { invoke } = window.__TAURI__.tauri;

let greetInputEl;
let updateMsg;

let state;
let details;

async function updateStatus() {
	// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
	updateMsg.textContent = await invoke("update_status", { 
		stateMsg: state.value,
		detailsMsg: details.value,
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

