<script lang="ts">
  import type { Map } from "maplibre-gl";
  import { onDestroy, onMount } from "svelte";
  import { state } from "./stores";

  export let mode: Mode;
  export let app: LTN;
  export let prevMode: Mode;
  export let map: Map;

  // Initially set this
  $state = { state: "neutral" };

  onMount(() => {
    map.keyboard.disable();
  });
  onDestroy(() => {
    map.keyboard.enable();
  });

  function onKeyDown(e: KeyboardEvent) {
    if ($state.state == "chose-road") {
      if (e.key == "ArrowLeft" && $state.shortcutIndex != 0) {
        e.stopPropagation();
        $state.shortcutIndex--;
      }
      if (e.key == "ArrowRight") {
        e.stopPropagation();
        if ($state.shortcutIndex == null) {
          $state.shortcutIndex = 0;
        } else if ($state.shortcutIndex != $state.gj.features.length - 1) {
          $state.shortcutIndex++;
        }
      }
    }
  }

  function back() {
    mode = prevMode;
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<div><button on:click={back}>Back to editing</button></div>

{#if $state.state == "neutral"}
  <p>Click a road to see shortcuts</p>
{:else if $state.state == "chose-road"}
  <div>
    <button
      disabled={$state.shortcutIndex == null || $state.shortcutIndex == 0}
      on:click={() => $state.shortcutIndex--}
    >
      Prev
    </button>
    {$state.shortcutIndex} / {$state.gj.features.length}
    <button
      disabled={$state.shortcutIndex == $state.gj.features.length - 1}
      on:click={() => $state.shortcutIndex++}
    >
      Next
    </button>
  </div>
{/if}
