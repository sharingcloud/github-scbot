<script lang="ts">
	/** @restProps div */
</script>

<div class={`app-card ${$$restProps.class}`} {...$$restProps}>
	{#if $$slots.header}
		<div class="app-card__header">
			<slot name="header" />
		</div>
	{/if}

	<div class="app-card__content">
		<slot />
	</div>

	{#if $$slots.footer}
		<div class="app-card__footer">
			<slot name="footer" />
		</div>
	{/if}
</div>
