<script lang="ts">
  import { downloadGeneratedFile, Loading } from "./common";
  import { app, mode } from "./stores";

  export let example: string;

  let msg: string | null = null;

  // TODO Could split this stuff; it just cares about the example
  function saveGj() {
    let filename = example || "custom";
    downloadGeneratedFile(`ltn_${filename}.geojson`, $app.toSavefile());
  }

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    try {
      loadEdits(await fileInput.files![0].text());
    } catch (err) {
      window.alert(`Couldn't open this file: ${err}`);
    }
    msg = null;
  }

  function loadEdits(gj: string) {
    msg = "Loading edits from file";
    // TODO If we're already in one of the states, nothing refreshes immediately...
    if ($app.loadSavefile(JSON.parse(gj))) {
      $mode = { mode: "neighbourhood" };
    } else {
      $mode = { mode: "network" };
    }
  }
</script>

{#if $app}
  <div><button on:click={saveGj}>Save to GJ</button></div>
  <div>
    <label>
      Load edits from GJ
      <input bind:this={fileInput} on:change={loadFile} type="file" />
    </label>
  </div>
{/if}

<Loading {msg} />
