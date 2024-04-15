<script lang="ts">
	import type { PullRequest } from "$api/types";
	import Card from "./base/Card.svelte";
	import DescriptionList from "./base/DescriptionList.svelte";
	import DescriptionListLine from "./base/DescriptionListLine.svelte";
	import DescriptionListDetails from "./base/DescriptionListDetails.svelte";
	import DescriptionListTerm from "./base/DescriptionListTerm.svelte";

	export let pullRequest: PullRequest;
</script>

<Card>
	<svelte:fragment slot="header">
		Pull request: #{pullRequest.number}
	</svelte:fragment>

	<DescriptionList>
		<DescriptionListLine>
			<DescriptionListTerm>ID</DescriptionListTerm>
			<DescriptionListDetails>{pullRequest.id}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>QA status</DescriptionListTerm>
			<DescriptionListDetails>{pullRequest.qa_status}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Needed reviewers count</DescriptionListTerm>
			<DescriptionListDetails>{pullRequest.needed_reviewers_count}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Status comment ID</DescriptionListTerm>
			<DescriptionListDetails>{pullRequest.status_comment_id}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Checks enabled</DescriptionListTerm>
			<DescriptionListDetails>{pullRequest.checks_enabled}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Automerge enabled</DescriptionListTerm>
			<DescriptionListDetails>{pullRequest.automerge}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Locked</DescriptionListTerm>
			<DescriptionListDetails>{pullRequest.locked}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Strategy override</DescriptionListTerm>
			<DescriptionListDetails>{pullRequest.strategy_override}</DescriptionListDetails>
		</DescriptionListLine>
	</DescriptionList>
</Card>
