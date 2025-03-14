<script lang="ts">
  import { Pencil, Trash2 } from "lucide-svelte";
  import { Loading } from "svelte-utils";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import CntChooseArea from "../CntChooseArea.svelte";
  import { Link } from "../common";
  import { routeTool } from "../common/draw_area/stores";
  import { appFocus, backend, currentProjectKey, map, mode } from "../stores";
  import {
    listProjects,
    loadProject,
    removeProject,
    renameProject,
  } from "./loader";
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
      let projectKey = params.get("project");
      if (projectKey) {
        loadProjectPrompt(projectKey);
      }
    } else {
      // Update the URL
      let url = new URL(window.location.href);
      url.searchParams.delete("project");
      window.history.replaceState(null, "", url.toString());
    }
  }

  let projectList = listProjects($appFocus);

  function deleteProjectPrompt(key: string) {
    if (window.confirm(`Really delete project ${key}? You can't undo this.`)) {
      removeProject(key);
      projectList = listProjects($appFocus);
    }
  }

  function renameProjectPrompt(key: string) {
    let newName = window.prompt(`Rename project ${key} to what?`, key);
    if (newName) {
      // TODO Again, hide leading ltn_?
      if (!newName.startsWith("ltn_")) {
        newName = `ltn_${newName}`;
      }

      renameProject(key, newName);
      projectList = listProjects($appFocus);
    }
  }

  async function loadProjectPrompt(key: string) {
    loading = `Loading project ${key}`;
    await loadProject(key);
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
            <h3 class="study-area-name">{study_area_name ?? "custom area"}</h3>
            <ul class="navigable-list">
              {#each projects as { projectId, projectName }}
                <li class="actionable-cell">
                  <h3>
                    <Link on:click={() => loadProject(projectId)}>
                      {projectName}
                    </Link>
                  </h3>
                  <span class="actions">
                    <button
                      class="outline icon-btn"
                      aria-label="Rename project"
                      on:click={() => renameProjectPrompt(projectId)}
                    >
                      <Pencil color="black" />
                    </button>
                    <button
                      class="icon-btn destructive"
                      aria-label="Delete project"
                      on:click={() => deleteProjectPrompt(projectId)}
                    >
                      <Trash2 color="white" />
                    </button>
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
  .study-area-name {
    border-bottom: 1px solid #444;
  }
  .project-list {
    margin-top: 18px;
  }
  .project-list li {
    padding-left: 1em;
  }
</style>
