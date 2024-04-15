<script lang="ts">
	import type { MergeRule } from "$lib/api/types";

	import Card from "./base/Card.svelte";
	import DescriptionList from "./base/DescriptionList.svelte";
	import DescriptionListLine from "./base/DescriptionListLine.svelte";
	import DescriptionListDetails from "./base/DescriptionListDetails.svelte";
	import DescriptionListTerm from "./base/DescriptionListTerm.svelte";

	export let mergeRule: MergeRule;
</script>

<Card>
	<svelte:fragment slot="header">
		Merge rule: {mergeRule.base_branch.toString()} &lt;- {mergeRule.head_branch.toString()}
	</svelte:fragment>

	<DescriptionList>
		<DescriptionListLine>
			<DescriptionListTerm>Base</DescriptionListTerm>
			<DescriptionListDetails>{mergeRule.base_branch.toString()}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Head</DescriptionListTerm>
			<DescriptionListDetails>{mergeRule.head_branch.toString()}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Strategy</DescriptionListTerm>
			<DescriptionListDetails>{mergeRule.strategy}</DescriptionListDetails>
		</DescriptionListLine>
	</DescriptionList>
</Card>
