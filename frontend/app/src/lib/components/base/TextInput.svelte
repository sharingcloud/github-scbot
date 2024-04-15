<script lang="ts">
	/** @restProps input */

	export let labelText: string | undefined = undefined;
	export let value: string | undefined = undefined;
</script>

<div class={`app-textinput ${$$restProps.class}`}>
	<label>{labelText}</label>
	<input type="text" class="app-textinput__input" bind:value {...$$restProps} />
</div>
