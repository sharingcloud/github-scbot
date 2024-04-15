<script lang="ts">
	/** @restProps button */
</script>

<button class={`app-button ${$$restProps.class}`} {...$$restProps} on:click>
	<slot />
</button>
