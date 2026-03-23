<script lang="ts">
	// Wizard uses its own full-screen layout
	// Don't render the main app header/footer wrapper
	let { children } = $props();
</script>

{@render children()}
