<script lang="ts">
	import { goto } from "$app/navigation";
	import { ApiClient } from "$api/client";
	import type { ExtendedExternalAccount } from "$api/types";
	import ExternalAccountCard from "$components/ExternalAccountCard.svelte";
	import Header from "$components/base/Header.svelte";
	import Button from "$components/base/Button.svelte";
	import { onMount } from "svelte";

	let extendedExternalAccounts: ExtendedExternalAccount[] = [];

	onMount(async () => {
		let client = new ApiClient();
		extendedExternalAccounts = await client.externalAccounts.list();
	});
</script>

<div>
	<Header>External accounts</Header>

	{#each extendedExternalAccounts as extendedExternalAccount}
		<ExternalAccountCard {extendedExternalAccount} />
	{/each}

	<Button on:click={() => goto(`/app/external-accounts/new`)}>Add new external account</Button>
</div>
