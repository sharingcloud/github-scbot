<script lang="ts">
	/** @restProps form */

	export let labelText: string | undefined = undefined;
</script>

<div class={`app-form ${$$restProps.class}`}>
	{#if labelText}
		<label>{labelText}</label>
	{/if}
	<form class="app-form__form" {...$$restProps} on:submit>
		<slot />
	</form>
</div>
