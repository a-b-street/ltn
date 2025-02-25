<script lang="ts">
  export let show = false;

  // Relies on external styling (picocss)
  let modalDialog: HTMLDialogElement | undefined;

  $: {
    if (modalDialog) {
      if (show) {
        modalDialog.showModal();
      } else {
        modalDialog.close();
      }
    }
  }

  function onClick(e: MouseEvent) {  
    // only dismiss the modal when clicking outside of the inner dialog content, on the dialog itself.
    if (e.target == modalDialog) {   
      show = false;
    }
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "Escape" || e.key == "Enter") {
      show = false;
    }
  }
</script>

<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
<dialog bind:this={modalDialog} on:click|stopPropagation={onClick} on:keydown={onKeyDown}>
  <article>
    <slot />
  </article>
</dialog>
