<script lang="ts">
  import { Link } from "./common";
  import { mode as storedMode, type Mode } from "./stores";

  export let mode: Mode;

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "Escape") {
      e.stopPropagation();
      $storedMode = mode;
    }
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<Link on:click={() => ($storedMode = mode)}>Back</Link>
