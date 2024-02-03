<script lang="ts">
  // It starts open
  // on:close is forwarded
  let dialog: HTMLDialogElement;

  function onClick(e: MouseEvent) {
    if (dialog.open) {
      // Anything inside the modal counts as the <div> or something inside that
      if (e.target == dialog) {
        dialog.close();
      }
    }
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "Escape" || e.key == "Enter") {
      e.stopPropagation();
      dialog.close();
    }
  }
</script>

<svelte:window on:click={onClick} on:keydown={onKeyDown} />

<dialog open on:close bind:this={dialog}>
  <article>
    <slot {dialog} />
  </article>
</dialog>
