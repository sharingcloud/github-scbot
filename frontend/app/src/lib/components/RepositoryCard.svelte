<script lang="ts">
	import type { ExtendedRepository, PullRequestRule } from "$lib/api/types";
	import PullRequestCard from "./PullRequestCard.svelte";
	import MergeRuleCard from "./MergeRuleCard.svelte";
	import DescriptionList from "$components/base/DescriptionList.svelte";
	import DescriptionListTerm from "$components/base/DescriptionListTerm.svelte";
	import DescriptionListDetails from "$components/base/DescriptionListDetails.svelte";
	import PullRequestRuleCard from "./PullRequestRuleCard.svelte";
	import Button from "$components/base/Button.svelte";
	import Card from "$components/base/Card.svelte";

	import { goto } from "$app/navigation";
	import { createEventDispatcher } from "svelte";
	import DescriptionListLine from "./base/DescriptionListLine.svelte";
	import Divider from "./base/Divider.svelte";

	export let extendedRepository: ExtendedRepository;

	const dispatch = createEventDispatcher<{
		"pull-request-rule:delete": PullRequestRule
	}>();

	$: repository = extendedRepository.repository;
</script>

<Card>
	<svelte:fragment slot="header">
		Repository: {repository.owner}/{repository.name}
	</svelte:fragment>

	<DescriptionList>
		<DescriptionListLine>
			<DescriptionListTerm>ID</DescriptionListTerm>
			<DescriptionListDetails>{repository.id}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Manual interaction</DescriptionListTerm>
			<DescriptionListDetails>{repository.manual_interaction}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Validation regex for pull request title</DescriptionListTerm>
			<DescriptionListDetails>"{repository.pr_title_validation_regex}"</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Default merge strategy to use</DescriptionListTerm>
			<DescriptionListDetails>{repository.default_strategy}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Default needed reviewers count</DescriptionListTerm>
			<DescriptionListDetails>{repository.default_needed_reviewers_count}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Default automerge activation</DescriptionListTerm>
			<DescriptionListDetails>{repository.default_automerge}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Default QA activation</DescriptionListTerm>
			<DescriptionListDetails>{repository.default_enable_qa}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Default checks activation</DescriptionListTerm>
			<DescriptionListDetails>{repository.default_enable_checks}</DescriptionListDetails>
		</DescriptionListLine>
	</DescriptionList>

	<Divider>
		Pull requests
	</Divider>

	{#each extendedRepository.pull_requests as pullRequest}
		<PullRequestCard {pullRequest} />
	{/each}

	<Button on:click={() => goto(`/app/repositories/${repository.id}/pull-requests/new`)} class="app-button">Add new pull request</Button>

	<Divider>
		Merge rules
	</Divider>

	{#each extendedRepository.merge_rules as mergeRule}
		<MergeRuleCard {mergeRule} />
	{/each}

	<Button on:click={() => goto(`/app/repositories/${repository.id}/merge-rules/new`)} class="app-button">Add new merge rule</Button>

	<Divider>
		Pull request rules
	</Divider>

	{#each extendedRepository.pull_request_rules as pullRequestRule}
		<PullRequestRuleCard {pullRequestRule} on:delete={(e) => dispatch("pull-request-rule:delete", e.detail)} />
	{/each}

	<Button on:click={() => goto(`/app/repositories/${repository.id}/pull-request-rules/new`)} class="app-button">Add new pull request rule</Button>
</Card>
