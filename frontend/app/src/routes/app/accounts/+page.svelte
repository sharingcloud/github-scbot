<script lang="ts">
	import { ApiClient } from "$api/client";
	import type { Account } from "$api/types";
	import AccountCard from "$components/AccountCard.svelte";
	import { onMount } from "svelte";
	import Button from "$components/base/Button.svelte";
	import Header from "$components/base/Header.svelte";
	import { goto } from "$app/navigation";

	let accounts: Account[] = [];

	onMount(async () => {
		let client = new ApiClient();
		accounts = await client.accounts.list();
	});
</script>

<div>
	<Header>Accounts</Header>

	{#each accounts as account}
		<AccountCard {account} />
	{/each}

	<Button on:click={() => goto(`/app/accounts/new`)} class="app-button">Add new account</Button>
</div>
