<script lang="ts">
	import Button from "$components/base/Button.svelte";
	import type { PullRequestRule } from "$api/types";
	import { createEventDispatcher } from "svelte";

	import Card from "./base/Card.svelte";
	import DescriptionList from "./base/DescriptionList.svelte";
	import DescriptionListLine from "./base/DescriptionListLine.svelte";
	import DescriptionListDetails from "./base/DescriptionListDetails.svelte";
	import DescriptionListTerm from "./base/DescriptionListTerm.svelte";

	export let pullRequestRule: PullRequestRule;

	const dispatch = createEventDispatcher<{
		delete: PullRequestRule
	}>();
</script>

<Card>
	<svelte:fragment slot="header">
		Pull request rule: {pullRequestRule.name}
	</svelte:fragment>

	<DescriptionList>
		<DescriptionListLine>
			<DescriptionListTerm>Conditions</DescriptionListTerm>
			<DescriptionListDetails>{JSON.stringify(pullRequestRule.conditions, null, 2)}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Actions</DescriptionListTerm>
			<DescriptionListDetails>{JSON.stringify(pullRequestRule.actions, null, 2)}</DescriptionListDetails>
		</DescriptionListLine>
	</DescriptionList>

	<svelte:fragment slot="footer">
		<Button on:click={() => dispatch("delete", pullRequestRule)}>Delete</Button>
	</svelte:fragment>
</Card>
