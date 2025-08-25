<script lang="ts">
  import { Link } from "./common";
  import { mode as storedMode, type Mode } from "./stores";

  interface Props {
    mode: Mode;
  }

  let { mode }: Props = $props();

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "Escape") {
      e.stopPropagation();
      $storedMode = mode;
    }
  }
</script>

<svelte:window onkeydown={onKeyDown} />

<Link onclick={() => ($storedMode = mode)}>Back</Link>
