<script lang="ts">
  export let mode: Mode;
  export let app: LTN;
  export let setBoundaryMode;

  // TODO Ideally all of this can live in the same component as the map, so we
  // don't have to plumb stuff around and trigger events back and forth
  function onKeyDown(e: KeyboardEvent) {
    if (mode.mode == "neighbourhood") {
      if (e.key == "a" && !mode.addingFilter) {
        mode.addingFilter = true;
      }
      if (e.key == "z" && e.ctrlKey) {
        undo();
      }
      if (e.key == "y" && e.ctrlKey) {
        redo();
      }
    }
  }
  function undo() {
    app.undo();
    mode.rerender++;
  }
  function redo() {
    app.redo();
    mode.rerender++;
  }

  function reset() {
    mode = {
      mode: "network",
    };
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<div><button on:click={reset}>Reset</button></div>
<div><button on:click={setBoundaryMode}>Edit boundary</button></div>
<div>
  <button
    on:click={() => (mode.addingFilter = true)}
    disabled={mode.addingFilter}>Add a modal filter</button
  >
</div>
<div>
  <button on:click={() => (mode = { mode: "view-shortcuts", prevMode: mode })}
    >View shortcuts</button
  >
</div>

<div>
  <button disabled={mode.undoLength == 0} on:click={undo}>
    {#if mode.undoLength == 0}
      Undo
    {:else}
      Undo ({mode.undoLength})
    {/if}
  </button>
  <button disabled={mode.redoLength == 0} on:click={redo}>
    {#if mode.redoLength == 0}
      Redo
    {:else}
      Redo ({mode.redoLength})
    {/if}
  </button>
</div>
