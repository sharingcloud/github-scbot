<script lang="ts">
	import { ApiClient } from "$api/client";
	import type { ExtendedRepository } from "$api/types";
	import RepositoryCard from "$components/RepositoryCard.svelte";
	import Header from "$components/base/Header.svelte";
	import { onMount } from "svelte";

	let repositories: ExtendedRepository[] = [];

	onMount(async () => {
		let client = new ApiClient();
		repositories = await client.repositories.list();
	});
</script>

<div>
	<Header>Repositories</Header>

	{#each repositories as extendedRepository}
		<RepositoryCard {extendedRepository} on:pull-request-rule:delete={async (e) => {
			let client = new ApiClient();
			await client.repositories.withId(extendedRepository.repository.id).pullRequestRules.delete(e.detail);
			repositories = await client.repositories.list();
		}} />
	{/each}
</div>
