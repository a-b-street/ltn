<script lang="ts">
  import { Link } from "../common";
  import SplitComponent from "../SplitComponent.svelte";
  import { app, projectName, map, mode, route_tool } from "../stores";
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
          <li>
            <Link on:click={() => loadFromLocalStorage(project)}>
              {project}
            </Link>
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
