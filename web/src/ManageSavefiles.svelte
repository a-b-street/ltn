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
    msg = "Loading project from file";
    try {
      loadProject(await fileInput.files![0].text());
    } catch (err) {
      window.alert(`Couldn't open this file: ${err}`);
    }
    msg = null;
  }

  function loadLocalStorage() {
    msg = "Loading projects from local storage";
    try {
      let gj = window.localStorage.getItem(filename);
      if (gj) {
        loadProject(gj);
      } else {
        window.alert("Nothing was saved in local storage");
      }
    } catch (err) {
      window.alert(`Couldn't load from local storage: ${err}`);
    }
    msg = null;
  }

  function saveLocalStorage() {
    window.localStorage.setItem(filename, $app!.toSavefile());
    window.alert("Saved");
  }

  function loadProject(gj: string) {
    $app!.loadSavefile(JSON.parse(gj));
    $mode = { mode: "network" };
    // Force a refresh in that mode maybe
    $app = $app;
  }
</script>

<details>
  <summary>Save / load project</summary>
  <button on:click={saveGj}>Save to GJ</button>
  <label>
    Load edits from GJ
    <input bind:this={fileInput} on:change={loadFile} type="file" />
  </label>
  <div style="display: flex; justify-content: space-between;">
    <button on:click={saveLocalStorage}>Save to local storage</button>
    <button on:click={loadLocalStorage}>Load from local storage</button>
  </div>
</details>

<Loading {msg} />
