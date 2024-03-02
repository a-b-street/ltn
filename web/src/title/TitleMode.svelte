<script lang="ts">
  import deleteLight from "../../assets/delete_light.svg?url";
  import deleteDark from "../../assets/delete_dark.svg?url";
  import editLight from "../../assets/edit_light.svg?url";
  import editDark from "../../assets/edit_dark.svg?url";
  import { Link } from "../common";
  import SplitComponent from "../SplitComponent.svelte";
  import {
    lightMode,
    app,
    projectName,
    map,
    mode,
    route_tool,
  } from "../stores";
  import { loadFromLocalStorage } from "./loader";

  export let wasmReady: boolean;

  // When other modes reset here, they can't clear state without a race condition
  $app = null;
  $route_tool = null;
  $projectName = "";

  let projectList = getProjectList();

  function getProjectList(): string[] {
    let list = [];
    for (let i = 0; i < window.localStorage.length; i++) {
      let key = window.localStorage.key(i)!;
      if (key.startsWith("ltn_")) {
        list.push(key);
      }
    }
    list.sort();
    return list;
  }

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    // TODO Be careful with overwriting stuff, leading ltn_, etc
    let key = "ltn_" + fileInput.files![0].name;
    window.localStorage.setItem(key, await fileInput.files![0].text());
    projectList = getProjectList();
    await loadFromLocalStorage(key);
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

      let gj = window.localStorage.getItem(key);
      window.localStorage.setItem(newName, gj);
      window.localStorage.removeItem(key);
      projectList = getProjectList();
    }
  }
</script>

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
        {#each projectList as project}
          <li style="display: flex; justify-content: space-between;">
            <Link on:click={() => loadFromLocalStorage(project)}>
              {project}
            </Link>
            <button class="secondary" on:click={() => renameProject(project)}>
              <img
                src={$lightMode ? editLight : editDark}
                alt="Rename project"
              />
            </button>
            <button class="secondary" on:click={() => deleteProject(project)}>
              <img
                src={$lightMode ? deleteLight : deleteDark}
                alt="Delete project"
              />
            </button>
          </li>
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
