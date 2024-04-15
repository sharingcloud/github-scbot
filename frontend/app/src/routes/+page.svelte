<script lang="ts">
	import { authToken } from "../lib/store";
	import { goto } from "$app/navigation";
	import { browser } from "$app/environment";

	if (browser) {
		if ($authToken == undefined) {
			goto("/signin");
		} else {
			goto("/app");
		}
	}
</script>

Loading...
