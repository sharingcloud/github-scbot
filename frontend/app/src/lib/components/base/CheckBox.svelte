<script lang="ts">
	/** @restProps input */

	export let labelText: string | undefined = undefined;
	export let checked: boolean | undefined = undefined;
</script>

<div class={`app-checkbox ${$$restProps.class}`}>
	<label>{labelText}</label>
	<input type="checkbox" class="app-checkbox__input" bind:checked {...$$restProps} />
</div>
