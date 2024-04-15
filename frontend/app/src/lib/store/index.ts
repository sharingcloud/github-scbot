import { browser } from "$app/environment";
import { writable } from "svelte/store";

function createAuthTokenStore() {
	const token = browser && localStorage.getItem("PRBOT_ADMIN_ACCESS_TOKEN") || null;
	const { subscribe, set } = writable<string | null>(token);

	return {
		subscribe,
		set: (value: string) => {
			localStorage.setItem("PRBOT_ADMIN_ACCESS_TOKEN", value);
			set(value)
		},
		reset: () => {
			localStorage.removeItem("PRBOT_ADMIN_ACCESS_TOKEN");
			set("");
		}
	}
}

export const authToken = createAuthTokenStore();
