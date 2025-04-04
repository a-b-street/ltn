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
    <span class="page-title">{pageTitle(mode.mode)}</span>
  </slot>
</Link>

<style>
  .page-title {
    display: inline-block;
    max-width: 20vw;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin: 0;
    vertical-align: middle;
  }
</style>
