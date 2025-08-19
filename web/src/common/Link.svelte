<script lang="ts">
  import type { Snippet } from "svelte";

  export let children: Snippet;
</script>

<!-- svelte-ignore a11y-invalid-attribute -->
<a href="#" on:click|preventDefault>
  {@render children()}
</a>
