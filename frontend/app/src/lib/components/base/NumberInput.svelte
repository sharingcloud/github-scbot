<script lang="ts">
	/** @restProps input */

	export let labelText: string | undefined = undefined;
	export let value: string | undefined = undefined;
</script>

<div class={`app-numberinput ${$$restProps.class}`}>
	<label>{labelText}</label>
	<input type="number" class="app-numberinput__input" bind:value {...$$restProps} />
</div>
