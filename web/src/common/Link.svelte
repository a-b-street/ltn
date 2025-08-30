<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
    onclick: () => void;
  }
  let { children, onclick }: Props = $props();

  function preventDefault(cb: () => void): (e: Event) => void {
    return function (ev: Event) {
      ev.preventDefault();
      cb();
    };
  }
</script>

<!-- svelte-ignore a11y_invalid_attribute -->
<a href="#" onclick={preventDefault(onclick)}>
  {@render children()}
</a>
