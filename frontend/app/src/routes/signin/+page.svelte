<script lang="ts">
	import Button from "$components/base/Button.svelte";
	import TextArea from "$components/base/TextArea.svelte";
	import { authToken } from "$store";
	import { goto } from "$app/navigation";

	let token = "";

	function saveToken() {
		authToken.set(token);
		goto("/app")
	}
</script>

<div class="app-signin-page">
	<div class="app-signin-page__container">
		<h4>Enter your Admin Access Token</h4>

		<TextArea labelText="Admin Access Token" placeholder="Enter your token..." bind:value={token} />
		<Button on:click={() => saveToken()}>Save</Button>
	</div>
</div>
