<script lang="ts">
  import { Pencil, Trash2 } from "lucide-svelte";
  import { Loading } from "svelte-utils";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import CntChooseArea from "../CntChooseArea.svelte";
  import { Link } from "../common";
  import { routeTool } from "../common/draw_area/stores";
  import { appFocus, backend, currentProjectKey, map, mode } from "../stores";
  import { getProjectList, loadFromLocalStorage } from "./loader";
  import LoadSavedProject from "./LoadSavedProject.svelte";

  export let wasmReady: boolean;
  export let firstLoad: boolean;

  let loading = "";

  // When other modes reset here, they can't clear state without a race condition
  {
    $backend = null;
    $routeTool = null;
    $currentProjectKey = "";

    if (firstLoad) {
      let params = new URLSearchParams(window.location.search);
      let loadProject = params.get("project");
      if (loadProject) {
        loading = `Loading project ${loadProject}`;
        loadFromLocalStorage(loadProject);
      }
    } else {
      // Update the URL
      let url = new URL(window.location.href);
      url.searchParams.delete("project");
      window.history.replaceState(null, "", url.toString());
    }
  }

  let projectList = getProjectList($appFocus);

  function deleteProject(key: string) {
    if (window.confirm(`Really delete project ${key}? You can't undo this.`)) {
      window.localStorage.removeItem(key);
      projectList = getProjectList($appFocus);
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
      projectList = getProjectList($appFocus);
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
      {#if projectList.length > 0}
        <h2>Your projects</h2>
        <div class="project-list">
          {#each projectList as [study_area_name, projects]}
            <h3 class="study-area">{study_area_name ?? "custom area"}</h3>
            <ul class="study-area-project-list">
              {#each projects as { projectId, projectName }}
                <li>
                  <span
                    style="display: flex; gap: 16px; justify-content: space-between;"
                  >
                    <Link on:click={() => loadProject(projectId)}>
                      {projectName}
                    </Link>
                    <span style="display: flex; gap: 16px;">
                      <button
                        class="outline icon-btn"
                        aria-label="Rename project"
                        on:click={() => renameProject(projectId)}
                      >
                        <Pencil color="black" />
                      </button>
                      <button
                        class="icon-btn destructive"
                        aria-label="Delete project"
                        on:click={() => deleteProject(projectId)}
                      >
                        <Trash2 color="white" />
                      </button>
                    </span>
                  </span>
                </li>
              {/each}
            </ul>
          {/each}
        </div>
      {/if}

      <h2>Start a new project</h2>
      {#if $appFocus == "global"}
        <button on:click={() => ($mode = { mode: "new-project" })}>
          New project
        </button>
      {:else if $appFocus == "cnt"}
        <CntChooseArea bind:activityIndicatorText={loading} />
      {/if}
      <LoadSavedProject bind:loading />
    {:else}
      <p>Waiting for MapLibre and WASM to load...</p>
    {/if}
  </div>
</SplitComponent>

<style>
  h2 {
    font-size: 32px;
  }

  .project-list h3.study-area {
    font-size: 20px;
    padding: 4px;
    margin: 4px 0;
    border-bottom: 1px solid #444;
  }

  .study-area-project-list {
    padding: 0 8px 0 4px;
    margin: 0;
    margin-bottom: 16px;
  }

  .study-area-project-list li {
    list-style-type: none;
    margin: 0;
    margin-left: 1em;
    padding-top: 4px;
    padding-bottom: 4px;
  }

  .study-area-project-list li:not(:last-child) {
    border-bottom: 1px solid #ddd;
  }
</style>
