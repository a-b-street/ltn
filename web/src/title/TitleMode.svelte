<script lang="ts">
  import { FileDown, Pencil, Trash2 } from "lucide-svelte";
  import { Loading } from "svelte-utils";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import CntChooseArea from "../CntChooseArea.svelte";
  import {
    downloadProject,
    Link,
    pageTitle,
    prettyPrintStudyAreaName,
  } from "../common";
  import { routeTool } from "../common/draw_area/stores";
  import { type ProjectID } from "../common/ProjectStorage";
  import {
    appFocus,
    backend,
    currentProjectID,
    map,
    mode,
    projectStorage,
  } from "../stores";
  import { loadProject } from "./loader";
  import LoadSavedProject from "./LoadSavedProject.svelte";

  export let wasmReady: boolean;
  export let firstLoad: boolean;

  let loading = "";

  // When other modes reset here, they can't clear state without a race condition
  {
    $backend = null;
    $routeTool = null;
    $currentProjectID = undefined;

    if (firstLoad) {
      let params = new URLSearchParams(window.location.search);
      let projectID = params.get("project") as ProjectID;
      if (projectID) {
        loadProjectFromUrlParam(projectID);
      }
    } else {
      // Update the URL
      let url = new URL(window.location.href);
      url.searchParams.delete("project");
      window.history.replaceState(null, "", url.toString());
    }
  }

  let studyAreas = $projectStorage.studyAreaProjects();

  function loadProjectFromUrlParam(projectIDParam: string) {
    let projectID = projectIDParam as ProjectID;
    try {
      let projectName = $projectStorage.projectName(projectID);
      if (!projectName) {
        console.error(`Project ${projectID} from URL not found`);
        return;
      }
      projectLoadingScreen(projectID, projectName);
    } catch {
      console.error(`Error trying to fetch project from URL: ${projectID}`);
    }
  }

  function deleteProject(projectID: ProjectID, projectName: string) {
    if (
      window.confirm(
        `Really delete project ${projectName}? You can't undo this.`,
      )
    ) {
      $projectStorage.removeProject(projectID);
      studyAreas = $projectStorage.studyAreaProjects();
    }
  }

  function renameProject(projectID: ProjectID, existingName: string) {
    let newName = window.prompt(
      `Rename project ${existingName} to what?`,
      existingName,
    );
    if (newName) {
      try {
        $projectStorage.renameProject(projectID, newName);
      } catch (e) {
        window.alert(`Couldn't rename project: ${e}`);
      }
      studyAreas = $projectStorage.studyAreaProjects();
    }
  }

  async function projectLoadingScreen(
    projectID: ProjectID,
    projectName: string,
  ) {
    loading = `Loading project ${projectName}`;
    await loadProject(projectID);
    $mode = { mode: "pick-neighbourhood" };
    loading = "";
  }
</script>

<Loading {loading} />

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>{pageTitle($mode.mode)}</li>
      </ul>
    </nav>
  </div>
  <div slot="sidebar">
    {#if $map && wasmReady}
      {#if studyAreas.length > 0}
        <h1>Your projects</h1>
        <div class="project-list">
          {#each studyAreas as [studyAreaName, projects]}
            <h2 class="study-area-name">
              {prettyPrintStudyAreaName(studyAreaName)}
            </h2>
            <ul class="navigable-list">
              {#each projects as { projectID, projectName }}
                <li class="actionable-cell">
                  <h3>
                    <Link
                      on:click={() =>
                        projectLoadingScreen(projectID, projectName)}
                    >
                      {projectName}
                    </Link>
                  </h3>
                  <span class="actions">
                    <button
                      class="outline icon-btn"
                      title="Download project as GeoJSON"
                      on:click={() => downloadProject(projectID)}
                    >
                      <FileDown color="black" />
                    </button>
                    <button
                      class="outline icon-btn"
                      title="Rename project"
                      on:click={() => renameProject(projectID, projectName)}
                    >
                      <Pencil color="black" />
                    </button>
                    <button
                      class="icon-btn destructive"
                      title="Delete project"
                      on:click={() => deleteProject(projectID, projectName)}
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

      <h1>Start a new project</h1>
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
