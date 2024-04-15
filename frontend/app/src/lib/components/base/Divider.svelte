<script lang="ts">
	/** @restProps div */

	import clsx from "clsx";
</script>

<div class={clsx("app-divider", $$slots.default && "app-divider--header", !$$slots.default && "app-divider--no-header")}>
	<div class="app-divider__divider">
		<slot />
	</div>
</div>
