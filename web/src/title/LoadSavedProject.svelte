<script lang="ts">
  import { stripPrefix, stripSuffix } from "../common";
  import { loadFromLocalStorage } from "./loader";

  export let loading: string;

  let fileInput: HTMLInputElement;

  async function loadFile(e: Event) {
    let filename = fileInput.files![0].name;
    loading = `Loading from file ${filename}`;

    let contents = await fileInput.files![0].text();
    let gj = JSON.parse(contents);
    // Is this a CNT project or a regular one?
    let key = "";
    if (gj.study_area_name && gj.study_area_name.startsWith("LAD_")) {
      let kind = "ltn_cnt";
      // Parse the project name from the filename, best effort. The user may
      // have renamed the file.
      let projectName = stripSuffix(
        stripPrefix(filename, `${kind}_${gj.study_area_name}_`),
        ".geojson",
      );
      key = `${kind}/${gj.study_area_name}/${projectName}`;
    } else {
      let projectName = stripSuffix(stripPrefix(filename, "ltn_"), ".geojson");
      key = `ltn_${projectName}`;
    }
    // TODO Be careful with overwriting files
    window.localStorage.setItem(key, contents);
    await loadFromLocalStorage(key);
    loading = "";
  }
</script>

<label>
  Load a project from a file
  <input bind:this={fileInput} on:change={loadFile} type="file" />
</label>
