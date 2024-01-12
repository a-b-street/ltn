<script lang="ts">
  // It starts open
  // on:close is forwarded
  let dialog;
</script>

<dialog open on:close bind:this={dialog}>
  <div>
    <slot {dialog} />
  </div>
</dialog>

<style>
  dialog {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 999;
  }

  div {
    width: 80%;
    height: 80%;
    background: white;
  }
</style>
