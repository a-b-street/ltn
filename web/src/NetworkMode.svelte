<script lang="ts">
  import type { Feature } from "geojson";
  import { FillLayer, GeoJSON, hoverStateFilter, Popup } from "svelte-maplibre";
  import { notNull } from "./common";
  import ManageSavefiles from "./ManageSavefiles.svelte";
  import ModalFilterLayer from "./ModalFilterLayer.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode } from "./stores";

  // Note we do this to trigger a refresh when loading stuff
  $: gj = JSON.parse(notNull($app).toSavefile());
  $: boundaryNames = gj.features
    .filter((f: Feature) => notNull(f.properties).kind == "boundary")
    .map((f: Feature) => notNull(f.properties).name);

  function resetTitle() {
    $mode = { mode: "title" };
    $app = null;
    // TODO If we were being paranoid, route_tool as well
  }

  function pickNeighbourhood(name: string) {
    $app!.setCurrentNeighbourhood(name);
    $mode = { mode: "neighbourhood" };
  }

  function deleteNeighbourhood(name: string) {
    $app!.deleteNeighbourhoodBoundary(name);
    gj = JSON.parse(notNull($app).toSavefile());
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
  <div slot="sidebar">
    <h1>Define neighbourhood boundaries</h1>
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

    <div>
      <button on:click={resetTitle}
        >Start over and change your study area</button
      >
    </div>
    <div>
      <button on:click={newBoundary}>Draw a new boundary</button>
    </div>
    {#each boundaryNames as name}
      <div>
        <button on:click={() => pickNeighbourhood(name)}>{name}</button>
        <button on:click={() => deleteNeighbourhood(name)}>X</button>
      </div>
    {/each}

    <hr />
    <ManageSavefiles />
  </div>

  <div slot="map">
    <GeoJSON data={gj} generateId>
      <FillLayer
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
        <Popup openOn="hover" let:data>
          <p>{notNull(data).properties.name}</p>
        </Popup>
      </FillLayer>
      <ModalFilterLayer />
    </GeoJSON>
  </div>
</SplitComponent>
