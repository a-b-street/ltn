<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Mode } from "../stores";
  import { mode as storedMode } from "../stores";
  import Link from "./Link.svelte";
  import { pageTitle } from "./navbar";

  interface Props {
    mode: Mode;
    children?: Snippet | undefined;
  }

  let { mode, children = undefined }: Props = $props();
</script>

<Link onclick={() => ($storedMode = mode)}>
  {#if children}
    {@render children()}
  {:else}
    <span class="page-title">{pageTitle(mode.mode)}</span>
  {/if}
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
