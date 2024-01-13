<script lang="ts">
  import { downloadGeneratedFile, Loading } from "./common";
  import { app, example, mode } from "./stores";

  $: filename = `ltn_${$example || "custom"}.geojson`;

  let msg: string | null = null;

  function saveGj() {
    downloadGeneratedFile(filename, $app!.toSavefile());
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
    msg = "Loading edits from file or local storage";
    // TODO If we're already in one of the states, nothing refreshes immediately...
    if ($app!.loadSavefile(JSON.parse(gj))) {
      $mode = { mode: "neighbourhood" };
    } else {
      $mode = { mode: "network" };
    }
    msg = null;
  }

  function loadLocalStorage() {
    let gj = window.localStorage.getItem(filename);
    if (gj) {
      loadEdits(gj);
    } else {
      window.alert("Nothing saved");
    }
  }

  function saveLocalStorage() {
    window.localStorage.setItem(filename, $app!.toSavefile());
  }
</script>

<details>
  <summary>Save / load project</summary>
  <div><button on:click={saveGj}>Save to GJ</button></div>
  <div>
    <label>
      Load edits from GJ
      <input bind:this={fileInput} on:change={loadFile} type="file" />
    </label>
  </div>
  <div>
    <button on:click={saveLocalStorage}>Save to local storage</button><button
      on:click={loadLocalStorage}>Load from local storage</button
    >
  </div>
</details>

<Loading {msg} />
