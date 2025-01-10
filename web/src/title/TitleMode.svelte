<script lang="ts">
  import { Loading } from "svelte-utils";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import deleteIcon from "../../assets/delete.svg?url";
  import editIcon from "../../assets/edit.svg?url";
  import { Link } from "../common";
  import { routeTool } from "../common/draw_area/stores";
  import { backend, map, mode, projectName } from "../stores";
  import { loadFromLocalStorage } from "./loader";

  export let wasmReady: boolean;
  export let firstLoad: boolean;

  let loading = "";

  // When other modes reset here, they can't clear state without a race condition
  {
    $backend = null;
    $routeTool = null;
    $projectName = "";

    if (firstLoad) {
      let params = new URLSearchParams(window.location.search);
      let loadProject = params.get("project");
      if (loadProject) {
        loadFromLocalStorage(loadProject);
      }
    } else {
      // Update the URL
      let url = new URL(window.location.href);
      url.searchParams.delete("project");
      window.history.replaceState(null, "", url.toString());
    }
  }

  let projectList = getProjectList();

  // Returns a list, grouped and sorted by the optional study_area_name, with
  // custom cases at the end
  function getProjectList(): Array<[string, string[]]> {
    let perArea = new Map();
    let custom = [];
    for (let i = 0; i < window.localStorage.length; i++) {
      let key = window.localStorage.key(i)!;
      if (key.startsWith("ltn_")) {
        let study_area_name = "";
        try {
          let gj = JSON.parse(window.localStorage.getItem(key)!);
          study_area_name = gj.study_area_name;
        } catch (err) {
          // Ignore it
        }
        if (study_area_name && study_area_name.length > 0) {
          if (!perArea.has(study_area_name)) {
            perArea.set(study_area_name, []);
          }
          perArea.get(study_area_name)!.push(key);
        } else {
          custom.push(key);
        }
      }
    }

    let out = [...perArea.entries()];
    out.sort((a, b) => a[0].localeCompare(b[0]));
    out.push(["custom", custom]);
    return out;
  }

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    // TODO Be careful with overwriting stuff, leading ltn_, etc
    let key = "ltn_" + fileInput.files![0].name;
    loading = `Loading from file ${key}`;
    window.localStorage.setItem(key, await fileInput.files![0].text());
    projectList = getProjectList();
    await loadFromLocalStorage(key);
    loading = "";
  }

  function deleteProject(key: string) {
    if (window.confirm(`Really delete project ${key}? You can't undo this.`)) {
      window.localStorage.removeItem(key);
      projectList = getProjectList();
    }
  }

  function renameProject(key: string) {
    let newName = window.prompt(`Rename project ${key} to what?`, key);
    if (newName) {
      // TODO Again, hide leading ltn_?
      if (!newName.startsWith("ltn_")) {
        newName = `ltn_${newName}`;
      }

      let gj = window.localStorage.getItem(key)!;
      window.localStorage.setItem(newName, gj);
      window.localStorage.removeItem(key);
      projectList = getProjectList();
    }
  }

  async function loadProject(key: string) {
    loading = `Loading project ${key}`;
    await loadFromLocalStorage(key);
    loading = "";
  }
</script>

<Loading {loading} />

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>Choose project</li>
      </ul>
    </nav>
  </div>
  <div slot="sidebar">
    {#if $map && wasmReady}
      <div>
        <Link on:click={() => ($mode = { mode: "new-project" })}>
          New project
        </Link>
      </div>

      <p>Load a saved project:</p>
      <ul>
        {#each projectList as [study_area_name, projects]}
          <u>{study_area_name ?? "custom area"}</u>
          {#each projects as project}
            <li>
              <span style="display: flex; justify-content: space-between;">
                <Link on:click={() => loadProject(project)}>
                  {project.slice("ltn_".length)}
                </Link>
                <button
                  class="secondary"
                  on:click={() => renameProject(project)}
                >
                  <img src={editIcon} alt="Rename project" />
                </button>
                <button
                  class="secondary"
                  on:click={() => deleteProject(project)}
                >
                  <img src={deleteIcon} alt="Delete project" />
                </button>
              </span>
            </li>
          {/each}
        {/each}
      </ul>

      <label>
        Load a project from a file
        <input bind:this={fileInput} on:change={loadFile} type="file" />
      </label>
    {:else}
      <p>Waiting for MapLibre and WASM to load...</p>
    {/if}
  </div>
</SplitComponent>
