<script lang="ts">
	import type { ExtendedExternalAccount } from "$api/types";

	import Card from "$components/base/Card.svelte";
	import DescriptionList from "$components/base/DescriptionList.svelte";
	import DescriptionListLine from "$components/base/DescriptionListLine.svelte";
	import DescriptionListDetails from "$components/base/DescriptionListDetails.svelte";
	import DescriptionListTerm from "$components/base/DescriptionListTerm.svelte";

	export let extendedExternalAccount: ExtendedExternalAccount;

	$: externalAccount = extendedExternalAccount.external_account;
</script>

<Card>
	<svelte:fragment slot="header">
		External account: {externalAccount.username}
	</svelte:fragment>

	<DescriptionList>
		<DescriptionListLine>
			<DescriptionListTerm>Private key</DescriptionListTerm>
			<DescriptionListDetails>{externalAccount.private_key}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Public key</DescriptionListTerm>
			<DescriptionListDetails>{externalAccount.public_key}</DescriptionListDetails>
		</DescriptionListLine>
		<DescriptionListLine>
			<DescriptionListTerm>Rights</DescriptionListTerm>
			<DescriptionListDetails>
				{#each extendedExternalAccount.rights as right}
					<div>#{right.repository_id}</div>
				{/each}
			</DescriptionListDetails>
		</DescriptionListLine>
	</DescriptionList>
</Card>
