<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { map } from "../stores";

  export let list: any[];
  export let idx = 0;

  onMount(() => {
    $map?.keyboard.disable();
  });
  onDestroy(() => {
    $map?.keyboard.enable();
  });

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "ArrowLeft") {
      e.stopPropagation();
      prev();
    }
    if (e.key == "ArrowRight") {
      e.stopPropagation();
      next();
    }
  }

  function prev() {
    if (idx != 0) {
      idx--;
    }
  }

  function next() {
    if (idx != list.length - 1) {
      idx++;
    }
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<div
  style="display: flex; justify-content: space-between; align-items: center;"
>
  <button disabled={idx == 0} on:click={prev} data-tooltip="Left">
    Previous
  </button>
  {idx + 1} / {list.length}
  <button
    disabled={idx == list.length - 1}
    on:click={next}
    data-tooltip="Right"
  >
    Next
  </button>
</div>
