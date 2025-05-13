<script lang="ts">
  import { FileDown, Pencil, Trash2 } from "lucide-svelte";
  import type { LngLatBoundsLike } from "maplibre-gl";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import cntBoundariesUrl from "../../assets/cnt_boundaries.geojson?url";
  import englandBoundariesUrl from "../../assets/england_boundaries.geojson?url";
  import ChooseArea from "../ChooseArea.svelte";
  import {
    downloadProject,
    Link,
    Loading,
    pageTitle,
    prettyPrintStudyAreaName,
  } from "../common";
  import { routeTool } from "../common/draw_area/stores";
  import { type ProjectID } from "../common/ProjectStorage";
  import {
    appFocus,
    backend,
    currentProjectID,
    firstTimeLoadProjectFromURL,
    map,
    mode,
    projectStorage,
  } from "../stores";
  import AppFocusSwitcher from "./AppFocusSwitcher.svelte";
  import { loadingMessage, loadingProgress, loadProject } from "./loader";
  import LoadSavedProject from "./LoadSavedProject.svelte";

  export let wasmReady: boolean;

  // When other modes reset here, they can't clear state without a race condition
  {
    $backend = null;
    $routeTool = null;
    $currentProjectID = undefined;

    if ($firstTimeLoadProjectFromURL) {
      let params = new URLSearchParams(window.location.search);
      let projectID = params.get("project") as ProjectID;
      if (projectID) {
        loadProjectFromUrlParam(projectID);
      }
      $firstTimeLoadProjectFromURL = false;
    } else {
      // Update the URL
      let url = new URL(window.location.href);
      url.searchParams.delete("project");
      window.history.replaceState(null, "", url.toString());
    }
  }

  {
    // The App component intiailizes the map to the proper zoom,
    // so no need to redundantly do it here for the first load.
    if ($map && !$firstTimeLoadProjectFromURL) {
      let bounds = [-180, -90, 180, 90] as LngLatBoundsLike;
      if ($appFocus == "cnt") {
        bounds = [-8.943, 54.631, -0.901, 59.489];
      } else if ($appFocus == "england") {
        bounds = [-5.96, 49.89, 2.31, 55.94];
      }
      $map.fitBounds(bounds, { duration: 500 });
    }
  }

  let studyAreas = $projectStorage.studyAreaProjects();

  function loadProjectFromUrlParam(projectIDParam: string) {
    let projectID = projectIDParam as ProjectID;
    try {
      openProject(projectID);
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

  async function openProject(projectID: ProjectID) {
    await loadProject(projectID);
    $mode = { mode: "pick-neighbourhood" };
  }
</script>

<Loading loading={$loadingMessage} progress={$loadingProgress} />

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
      {#if $appFocus == "cnt"}
        <h1>The Connected Neighbourhoods Tool</h1>
      {:else}
        <div style="display: flex; justify-content: space-between;">
          <h1>The LTN Tool</h1>
          <AppFocusSwitcher />
        </div>
      {/if}

      {#if studyAreas.length > 0}
        <h2>Your projects</h2>
        <div class="project-list">
          {#each studyAreas as [studyAreaName, projects]}
            <h3 class="study-area-name">
              {prettyPrintStudyAreaName(studyAreaName)}
            </h3>
            <ul class="navigable-list">
              {#each projects as { projectID, projectName }}
                <li class="actionable-cell">
                  <h3>
                    <Link on:click={() => openProject(projectID)}>
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

      <h2>Start a new project</h2>
      {#if $appFocus == "global"}
        <button on:click={() => ($mode = { mode: "new-project" })}>
          New project
        </button>
      {:else if $appFocus == "cnt"}
        <ChooseArea boundariesUrl={cntBoundariesUrl} />
      {:else if $appFocus == "england"}
        <ChooseArea boundariesUrl={englandBoundariesUrl} />
      {/if}
      <LoadSavedProject />
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
