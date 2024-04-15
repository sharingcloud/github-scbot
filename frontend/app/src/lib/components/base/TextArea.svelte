<script lang="ts">
	/** @restProps textarea */

	export let labelText: string | undefined = undefined;
</script>

<div class={`app-textarea ${$$restProps.class}`}>
	<label>{labelText}</label>
	<textarea class="app-textarea__textarea" {...$$restProps}>
		<slot />
	</textarea>
</div>
