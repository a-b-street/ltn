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
    msg = "Loading edits from file";
    try {
      loadEdits(await fileInput.files![0].text());
    } catch (err) {
      window.alert(`Couldn't open this file: ${err}`);
    }
    msg = null;
  }

  function loadLocalStorage() {
    msg = "Loading edits from local storage";
    try {
      let gj = window.localStorage.getItem(filename);
      if (gj) {
        loadEdits(gj);
      } else {
        window.alert("Nothing was saved");
      }
    } catch (err) {
      window.alert(`Couldn't load from local storage: ${err}`);
    }
    msg = null;
  }

  function saveLocalStorage() {
    window.localStorage.setItem(filename, $app!.toSavefile());
  }

  function loadEdits(gj: string) {
    $app!.loadSavefile(JSON.parse(gj));
    // TODO Make sure this refreshes if we're already there?
    $mode = { mode: "network" };
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
