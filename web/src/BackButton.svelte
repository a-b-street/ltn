<script lang="ts">
  import { Link } from "./common";
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

<Link on:click={() => dispatch("click")}>Back</Link>
