<script lang="ts">
  import { createBubbler, preventDefault } from 'svelte/legacy';

  const bubble = createBubbler();
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();
</script>

<!-- svelte-ignore a11y_invalid_attribute -->
<a href="#" onclick={preventDefault(bubble('click'))}>
  {@render children()}
</a>
