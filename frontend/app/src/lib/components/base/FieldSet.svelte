<script lang="ts">
	/** @restProps fieldset */

	export let labelText: string | undefined = undefined;
</script>

<div class={`app-fieldset ${$$restProps.class}`}>
	{#if labelText}
		<label>{labelText}</label>
	{/if}
	<fieldset class="app-fieldset__fieldset" {...$$restProps}>
		<slot />
	</fieldset>
</div>
