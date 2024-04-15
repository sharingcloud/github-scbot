<script lang="ts">
	/** @restProps select */

	export let labelText: string | undefined = undefined;
	export let value: string | undefined = undefined;
	export let options: Record<string, string> = {};
</script>

<div class={`app-select ${$$restProps.class}`}>
	<label>{labelText}</label>
	<select class="app-select__select" bind:value {...$$restProps}>
		{#each Object.entries(options) as [value, text]}
			<option value={value}>{text}</option>
		{/each}
	</select>
</div>
