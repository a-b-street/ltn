<script lang="ts">
  export let loading: string;
  export let progress: number | null = null;

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
    <div class="background">
      {loading}

      {#if progress != null}
        <div>
          {#if progress == 100}
            <progress style:width="100%" />
          {:else}
            <progress value={progress} max="100" style:width="100%" />
          {/if}
        </div>
      {/if}
    </div>
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

  .background {
    background: grey;
    padding: 16px;
  }
</style>
