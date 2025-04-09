<script lang="ts">
  export let loading;
  export let progress = null;

  let startTime: number | undefined = undefined;
  let taskName: string | undefined = undefined;
  $: {
    if (loading && !startTime) {
      // Start timing when loading begins
      startTime = Date.now();
      taskName = loading;
    } else if (!loading && startTime) {
      // Calculate and log duration when loading ends
      const duration = Date.now() - startTime;
      console.log(`Loading "${taskName}" took ${duration}ms`);
      startTime = undefined;
    }
  }
</script>

{#if loading}
  <div class="cover">
    {loading}

    {#if progress != null}
      <div>
        <progress value={progress} style:width="100%" />
      </div>
    {/if}
  </div>
{/if}

<style>
  .cover {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    z-index: 999;

    color: white;
    font-size: 32px;
  }
</style>
