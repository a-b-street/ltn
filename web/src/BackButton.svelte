<script lang="ts">
  import { createEventDispatcher } from "svelte";

  let dispatch = createEventDispatcher<{
    click: void;
  }>();

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "Escape") {
      e.stopPropagation();
      dispatch("click");
    }
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<button on:click={() => dispatch("click")}>Back</button>
