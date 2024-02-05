<script lang="ts">
  import type { Feature } from "geojson";
  import { FillLayer, GeoJSON, hoverStateFilter } from "svelte-maplibre";
  import { layerId, notNull, Popup } from "./common";
  import ManageSavefiles from "./ManageSavefiles.svelte";
  import ModalFilterLayer from "./ModalFilterLayer.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode } from "./stores";

  // Note we do this to trigger a refresh when loading stuff
  $: gj = JSON.parse($app!.toSavefile());
  $: boundaryNames = gj.features
    .filter((f: Feature) => f.properties!.kind == "boundary")
    .map((f: Feature) => f.properties!.name);

  function pickNeighbourhood(name: string) {
    $app!.setCurrentNeighbourhood(name);
    $mode = { mode: "neighbourhood" };
  }

  function deleteNeighbourhood(name: string) {
    $app!.deleteNeighbourhoodBoundary(name);
    gj = JSON.parse($app!.toSavefile());
  }

  function newBoundary() {
    let name = window.prompt("What do you want to name the neighbourhood?");
    if (name) {
      $mode = { mode: "set-boundary", name, existing: null };
    }
  }

  // TODO Also render filters here
  // TODO Hover on button and highlight on map
</script>

<SplitComponent>
  <div slot="top" style="display: flex; justify-content: space-between;">
    <nav aria-label="breadcrumb">
      <!-- svelte-ignore a11y-invalid-attribute -->
      <ul>
        <li>
          <a href="#" on:click={() => ($mode = { mode: "title" })}
            >Choose study area</a
          >
        </li>
        <li>Pick neighbourhood</li>
      </ul>
    </nav>
    <span style="display: flex">
      <button
        class="outline"
        style="margin-right: 8px"
        on:click={() => ($mode = { mode: "route", prevMode: "network" })}
        >Route</button
      >
      <button class="outline" on:click={() => ($mode = { mode: "debug-gj" })}
        >Debug route snapper</button
      >
    </span>
  </div>
  <div slot="sidebar">
    <p>
      Inside the neighbourhood you define, the goal is to eliminate (or
      deliberately permit) through-traffic. An appropriate neighbourhood
      boundary depends on many factors. The simplest approach is to find the
      area bounded on all sides by "main" roads, which are designed for higher
      traffic volumes. There are many other considerations, though -- maybe
      severances like rivers or rail should be part of a boundary. Bridges and
      tunnels near a boundary may be confusing as well. And note that your
      boundary may not match the conventional definition of "neighbourhood."
    </p>

    <button on:click={newBoundary}>Draw a new boundary</button>
    {#each boundaryNames as name}
      <div style="display: flex; justify-content: space-between;">
        <button class="outline" on:click={() => pickNeighbourhood(name)}
          >{name}</button
        >
        <button
          class="secondary outline"
          on:click={() => deleteNeighbourhood(name)}>X</button
        >
      </div>
    {/each}

    <hr />
    <ManageSavefiles />
  </div>

  <div slot="map">
    <GeoJSON data={gj} generateId>
      <FillLayer
        {...layerId("neighbourhood-boundaries")}
        filter={["==", ["get", "kind"], "boundary"]}
        paint={{
          "fill-color": "red",
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
      <ModalFilterLayer />
    </GeoJSON>
  </div>
</SplitComponent>
