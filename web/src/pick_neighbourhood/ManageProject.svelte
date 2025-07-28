<script lang="ts">
  import type { FeatureCollection } from "geojson";
  import { Copy, FileDown, FolderOpen } from "lucide-svelte";
  import { downloadGeneratedFile, notNull } from "svelte-utils";
  import { downloadProject, HelpButton, Link } from "../common";
  import { type ProjectID } from "../common/ProjectStorage";
  import {
    backend,
    currentProject,
    currentProjectID,
    devMode,
    mutationCounter,
    projectStorage,
  } from "../stores";

  export let projectGj: FeatureCollection;

  $: edits = countEdits(projectGj);

  let showPickProject = false;
  $: otherProjects = listOtherProjects(notNull($currentProjectID));

  let osmTimestamp = $backend!.getOsmTimestamp();

  function listOtherProjects(currentID: ProjectID): Array<{
    projectID: ProjectID;
    projectName: string;
  }> {
    for (let [studyAreaName, projects] of $projectStorage.studyAreaProjects()) {
      if (studyAreaName == $currentProject!.study_area_name) {
        return projects.filter((p) => p.projectID != currentID);
      }
    }
    return [];
  }

  function copyProject() {
    let existingName = $currentProject!.project_name;
    let newName = window.prompt(
      `Please name this copy of project ${existingName}`,
      $projectStorage.nextAvailableProjectName(existingName),
    );
    if (newName) {
      try {
        let newID = $projectStorage.copyProject($currentProjectID!, newName);
        // Open this new project. We don't need to swap out much state in the
        // backend or reset zoom or the usual things, since we've just
        // made an exact copy of something.
        $currentProjectID = newID;
        $backend!.changeProjectName(newName);

        // Update the URL
        let url = new URL(window.location.href);
        url.searchParams.set("project", newID);
        window.history.replaceState(null, "", url.toString());

        window.alert(`Done, you are now working on project ${newName}`);
      } catch (e) {
        window.alert(`Couldn't copy project: ${e}`);
      }
    }
  }

  function openProject(projectID: ProjectID) {
    showPickProject = false;

    // We don't need to change everything with the backend, metric buckets,
    // route snapper, etc -- so don't use loadProject.
    let project = $projectStorage!.project(projectID);
    $backend!.loadSavefile(project);
    $currentProjectID = projectID;
    $mutationCounter++;

    // Update the URL
    let url = new URL(window.location.href);
    url.searchParams.set("project", projectID);
    window.history.replaceState(null, "", url.toString());
  }

  function debugRouteSnapper() {
    downloadGeneratedFile(
      "debug_route_snapper.geojson",
      $backend!.toRouteSnapperGj(),
    );
  }

  function countEdits(gj: FeatureCollection): {
    modalFilters: number;
    deletedModalFilters: number;
    travelFlows: number;
  } {
    let modalFilters = 0;
    let deletedModalFilters = 0;
    let travelFlows = 0;
    for (let f of gj.features) {
      if (f.properties!.kind == "modal_filter") {
        modalFilters++;
      } else if (f.properties!.kind == "deleted_existing_modal_filter") {
        deletedModalFilters++;
      } else if (f.properties!.kind == "travel_flow") {
        travelFlows++;
      }
    }
    return { modalFilters, deletedModalFilters, travelFlows };
  }
</script>

<h2>Project: {notNull($currentProject).project_name}</h2>
<div style="display: flex; gap: 8px">
  <button
    class="outline"
    style="margin-right: 8px;"
    title="Download project as GeoJSON"
    on:click={() => downloadProject(notNull($currentProjectID))}
  >
    <div style="display: flex; align-items: center; gap: 8px; color: black;">
      <FileDown />
      <!-- 
            The text feels a little crowded aginst the right edge. 
            Currently this is the only place we use an icon+text button like this.
            But if we do more, we might want to pattern something out.
          -->
      <span style="margin-right: 2px;">Export</span>
    </div>
  </button>

  <button
    class="outline"
    style="margin-right: 8px;"
    title="Make a copy of this project"
    on:click={() => copyProject()}
  >
    <div style="display: flex; align-items: center; gap: 8px; color: black;">
      <Copy />
      <!-- 
            The text feels a little crowded aginst the right edge. 
            Currently this is the only place we use an icon+text button like this.
            But if we do more, we might want to pattern something out.
          -->
      <span style="margin-right: 2px;">Copy project</span>
    </div>
  </button>
</div>

{#if notNull($currentProject).study_area_name && otherProjects.length > 0}
  <details
    class="dropdown"
    style="display: inline-block; margin-top: 8px"
    bind:open={showPickProject}
  >
    <!-- svelte-ignore a11y-no-redundant-roles -->
    <summary role="button" class="contrast outline">
      <FolderOpen />
      <span style="margin-left: 2px;">Open another project</span>
    </summary>
    <ul>
      {#each otherProjects as item}
        <li>
          <Link on:click={() => openProject(item.projectID)}
            >{item.projectName}</Link
          >
        </li>
      {/each}
    </ul>
  </details>
{/if}

<p>
  {edits.modalFilters} new modal filter(s) added
</p>
<p>
  {edits.deletedModalFilters}
  existing modal filter(s) removed
</p>
<p>{edits.travelFlows} road segment direction(s) changed</p>

{#if osmTimestamp}
  <div style="display: flex">
    <i>Map data is from {osmTimestamp.toDateString()}</i>
    <HelpButton>
      <p>
        If this area has changed since then, please contact <a
          href="mailto:dabreegster@gmail.com">dabreegster@gmail.com</a
        > to use newer OpenStreetMap data.
      </p>
    </HelpButton>
  </div>
{/if}

{#if $devMode}
  <button class="secondary" on:click={debugRouteSnapper}>
    Debug route-snapper
  </button>
{/if}
