<script lang="ts">
  import type { Feature } from "geojson";
  import { FillLayer, GeoJSON, hoverStateFilter } from "svelte-maplibre";
  import { downloadGeneratedFile, notNull } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import deleteIcon from "../assets/delete.svg?url";
  import editIcon from "../assets/edit.svg?url";
  import { HelpButton, layerId, Link } from "./common";
  import { pickNeighbourhoodName } from "./common/pick_names";
  import ModalFilterLayer from "./ModalFilterLayer.svelte";
  import {
    autosave,
    backend,
    editPerimeterRoads,
    mode,
    projectName,
  } from "./stores";

  // Note we do this to trigger a refresh when loading stuff
  $: gj = $backend!.toSavefile();
  $: boundaryNames = gj.features
    .filter((f: Feature) => f.properties!.kind == "boundary")
    .map((f: Feature) => f.properties!.name);

  function pickNeighbourhood(name: string) {
    $backend!.setCurrentNeighbourhood(name, $editPerimeterRoads);
    $mode = { mode: "neighbourhood" };
  }

  function deleteNeighbourhood(name: string) {
    if (
      window.confirm(
        `Really delete neighbourhood ${name}? You can't undo this.`,
      )
    ) {
      $backend!.deleteNeighbourhoodBoundary(name);
      autosave();
      // TODO Improve perf, don't call this twice
      gj = $backend!.toSavefile();
    }
  }

  function renameNeighbourhood(name: string) {
    let newName = pickNeighbourhoodName(
      $backend!,
      `Rename neighbourhood ${name} to what?`,
      name,
    );
    if (newName) {
      $backend!.renameNeighbourhoodBoundary(name, newName);
      autosave();
      gj = $backend!.toSavefile();
    }
  }

  function newBoundary() {
    let name = pickNeighbourhoodName(
      $backend!,
      "What do you want to name the neighbourhood?",
      "",
    );
    if (name) {
      $mode = { mode: "set-boundary", name, existing: null };
    }
  }

  function exportGJ() {
    downloadGeneratedFile(
      $projectName + ".geojson",
      JSON.stringify($backend!.toSavefile()),
    );
  }

  function debugRouteSnapper() {
    downloadGeneratedFile(
      "debug_route_snapper.geojson",
      $backend!.toRouteSnapperGj(),
    );
  }

  // TODO Hover on button and highlight on map
</script>

<SplitComponent>
  <div slot="top" style="display: flex; justify-content: space-between;">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title", firstLoad: false })}>
            Choose project
          </Link>
        </li>
        <li>
          Pick neighbourhood
          <HelpButton>
            <p>
              Inside the neighbourhood you define, the goal is to eliminate (or
              deliberately permit) through-traffic. An appropriate neighbourhood
              boundary depends on many factors. The simplest approach is to find
              the area bounded on all sides by "main" roads, which are designed
              for higher traffic volumes. There are many other considerations,
              though -- maybe severances like rivers or rail should be part of a
              boundary. Bridges and tunnels near a boundary may be confusing as
              well. And note that your boundary may not match the conventional
              definition of "neighbourhood."
            </p>
          </HelpButton>
        </li>
      </ul>
    </nav>
    <nav>
      <ul>
        <li>
          <Link
            on:click={() => ($mode = { mode: "route", prevMode: "network" })}
          >
            Route
          </Link>
        </li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <div><Link on:click={newBoundary}>Draw a new boundary</Link></div>
    <div>
      <Link on:click={() => ($mode = { mode: "auto-boundaries" })}>
        Use an auto-generated boundary
      </Link>
    </div>

    <ul>
      {#each boundaryNames as name}
        <li>
          <span style="display: flex; justify-content: space-between;">
            <Link on:click={() => pickNeighbourhood(name)}>
              {name}
            </Link>
            <button
              class="secondary"
              on:click={() => renameNeighbourhood(name)}
            >
              <img src={editIcon} alt="Rename neighbourhood" />
            </button>
            <button
              class="secondary"
              on:click={() => deleteNeighbourhood(name)}
            >
              <img src={deleteIcon} alt="Delete neighbourhood" />
            </button>
          </span>
        </li>
      {/each}
    </ul>

    <hr />
    <p>Current project: {$projectName}</p>
    <button on:click={exportGJ}>Export project to GeoJSON</button>
    <button class="secondary" on:click={debugRouteSnapper}>
      Debug route-snapper
    </button>
  </div>

  <div slot="map">
    <GeoJSON data={gj} generateId>
      <FillLayer
        {...layerId("neighbourhood-boundaries", false)}
        filter={["==", ["get", "kind"], "boundary"]}
        paint={{
          "fill-color": "black",
          "fill-opacity": hoverStateFilter(0.3, 0.5),
        }}
        manageHoverState
        on:click={(e) =>
          pickNeighbourhood(notNull(e.detail.features[0].properties).name)}
        hoverCursor="pointer"
      >
        <Popup openOn="hover" let:props>
          <p>{props.name}</p>
        </Popup>
      </FillLayer>
    </GeoJSON>
    <ModalFilterLayer />
  </div>
</SplitComponent>
