<script lang="ts">
  import type { FeatureCollection, MultiPolygon, Polygon } from "geojson";
  import { onMount } from "svelte";
  import {
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { Popup } from "svelte-utils/map";
  import boundariesUrl from "../assets/cnt_boundaries.geojson?url";
  import { Link, prettyPrintStudyAreaName } from "./common";
  import { projectStorage } from "./stores";
  import { loadProject } from "./title/loader";

  export let activityIndicatorText: string;

  let gj: FeatureCollection<
    Polygon | MultiPolygon,
    { kind: "LAD"; name: string }
  > = {
    type: "FeatureCollection" as const,
    features: [],
  };
  let ladNames: string[] = [];

  onMount(async () => {
    let resp = await fetch(boundariesUrl);
    gj = await resp.json();

    for (let f of gj.features) {
      ladNames.push(f.properties.name);
    }
    ladNames.sort();
    ladNames = ladNames;
  });

  function onClick(e: CustomEvent<LayerClickInfo>) {
    let props = e.detail.features[0].properties!;
    newFile(`${props.kind}_${props.name}`);
  }

  async function newFile(studyAreaName: string) {
    let projectName = "";
    let created = false;
    while (!created) {
      projectName =
        window.prompt(
          `Please pick a project name to create in ${prettyPrintStudyAreaName(studyAreaName)}`,
          projectName,
        ) || "";
      if (projectName == "") {
        // If the user leaves this blank or presses cancel, stop prompting them.
        return;
      }
      activityIndicatorText = `Loading pre-clipped OSM area ${prettyPrintStudyAreaName(studyAreaName)}`;
      try {
        let projectID = $projectStorage.createEmptyProject(
          projectName,
          studyAreaName,
        );
        await loadProject(projectID);
        created = true;
      } catch (e) {
        window.alert(e);
      }
      activityIndicatorText = "";
    }
  }
</script>

<p>Choose a boundary below or on the map to begin sketching:</p>
<ul style="columns: 3">
  {#each ladNames as name}
    <li><Link on:click={() => newFile(`LAD_${name}`)}>{name}</Link></li>
  {/each}
</ul>

<GeoJSON data={gj} generateId>
  <FillLayer
    paint={{
      "fill-color": "rgb(200, 100, 240)",
      "fill-outline-color": "rgb(200, 100, 240)",
      "fill-opacity": hoverStateFilter(0.0, 0.5),
    }}
    beforeId="Road labels"
    manageHoverState
    hoverCursor="pointer"
    on:click={onClick}
  >
    <Popup openOn="hover" let:props>
      <p>Click to start a new project in {props.name}</p>
    </Popup>
  </FillLayer>

  <LineLayer
    paint={{
      "line-color": "rgb(200, 100, 240)",
      "line-width": 2.5,
    }}
    beforeId="Road labels"
    manageHoverState
  />
</GeoJSON>
