<script lang="ts">
  import type { FeatureCollection, MultiPolygon, Polygon } from "geojson";
  import { onMount } from "svelte";
  import {
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    Popup,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { prettyPrintStudyAreaName, Style } from "../common";
  import { mode, projectStorage } from "../stores";
  import { loadProject } from "./loader";

  export let boundariesUrl: string;

  // This component works for both CNT and England, because the boundaries for
  // both fit this format. We could consider generalizing in the future.
  let gj: FeatureCollection<
    Polygon | MultiPolygon,
    { kind: "LAD"; name: string }
  > = {
    type: "FeatureCollection" as const,
    features: [],
  };
  let ladNames: string[] = [];
  let ladChoice = "";

  onMount(async () => {
    let resp = await fetch(boundariesUrl);
    gj = await resp.json();

    for (let f of gj.features) {
      ladNames.push(f.properties.name);
    }
    ladNames.sort();
    ladNames = ladNames;
  });

  function onClick(e: LayerClickInfo) {
    let props = e.features[0].properties!;
    newFile(`${props.kind}_${props.name}`);
  }

  async function newFile(studyAreaName: string) {
    let projectName;
    let created = false;
    while (!created) {
      let defaultName: string | null =
        projectName ||
        $projectStorage.nextAvailableProjectName(
          prettyPrintStudyAreaName(studyAreaName),
        );
      projectName = window.prompt(
        `Please pick a project name to create in ${prettyPrintStudyAreaName(studyAreaName)}`,
        defaultName,
      );
      if (projectName === null) {
        // If the user presses cancel, stop prompting them.
        return;
      }
      projectName = projectName.trim();
      if (projectName === "") {
        window.alert(
          "Project name cannot be blank; Please pick a name or cancel.",
        );
        continue;
      }
      try {
        let projectID = $projectStorage.createEmptyProject(
          projectName,
          studyAreaName,
        );
        await loadProject(projectID);
        $mode = { mode: "add-neighbourhood" };
        created = true;
      } catch (e) {
        window.alert(e);
      }
    }
  }

  function chooseLAD() {
    if (ladChoice) {
      newFile(`LAD_${ladChoice}`);
    }
  }
</script>

<p>Choose a boundary below or on the map to begin:</p>
<select bind:value={ladChoice} onchange={chooseLAD} style="width: 90%">
  <option value=""></option>
  {#each ladNames as value}
    <option {value}>{value}</option>
  {/each}
</select>

<GeoJSON data={gj} generateId>
  <FillLayer
    paint={{
      "fill-color": Style.mapFeature.hover.backgroundColor,
      "fill-outline-color": Style.mapFeature.hover.backgroundColor,
      "fill-opacity": hoverStateFilter(0.0, 0.5),
    }}
    beforeId="Road labels"
    manageHoverState
    hoverCursor="pointer"
    onclick={onClick}
  >
    <Popup openOn="hover">
      {#snippet children({ data })}
        <p>Click to start a new project in {data!.properties!.name}</p>
      {/snippet}
    </Popup>
  </FillLayer>

  <LineLayer
    paint={{
      "line-color": Style.mapFeature.hover.backgroundColor,
      "line-width": 2.5,
    }}
    beforeId="Road labels"
    manageHoverState={false}
  />
</GeoJSON>
