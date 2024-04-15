<script lang="ts">
	import type { Account } from "$lib/api/types";

	import Card from "$components/base/Card.svelte";
	import DescriptionList from "$components/base/DescriptionList.svelte";
	import DescriptionListLine from "$components/base/DescriptionListLine.svelte";
	import DescriptionListDetails from "$components/base/DescriptionListDetails.svelte";
	import DescriptionListTerm from "$components/base/DescriptionListTerm.svelte";

	export let account: Account;
</script>

<Card>
	<svelte:fragment slot="header">
		Account: {account.username}
	</svelte:fragment>

	<DescriptionList>
		<DescriptionListLine>
			<DescriptionListTerm>Is admin</DescriptionListTerm>
			<DescriptionListDetails>{account.is_admin}</DescriptionListDetails>
		</DescriptionListLine>
	</DescriptionList>
</Card>
