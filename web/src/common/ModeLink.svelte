<script lang="ts">
  import type { Mode } from "../stores";
  import { mode as storedMode } from "../stores";
  import Link from "./Link.svelte";
  import { pageTitle } from "./navbar";

  export let mode: Mode;
  export let afterLink: (() => void) | undefined = undefined;
</script>

<Link
  on:click={() => {
    $storedMode = mode;
    afterLink && afterLink();
  }}
  ><slot>
    {pageTitle(mode.mode)}
  </slot>
</Link>
