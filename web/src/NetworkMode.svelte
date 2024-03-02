<script lang="ts">
  import deleteLight from "../assets/delete_light.svg?url";
  import deleteDark from "../assets/delete_dark.svg?url";
  import editLight from "../assets/edit_light.svg?url";
  import editDark from "../assets/edit_dark.svg?url";
  import { downloadGeneratedFile } from "./common";
  import type { Feature } from "geojson";
  import { FillLayer, GeoJSON, hoverStateFilter } from "svelte-maplibre";
  import { layerId, notNull, Popup, Link } from "./common";
  import ModalFilterLayer from "./ModalFilterLayer.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { lightMode, app, autosave, mode, projectName } from "./stores";

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
    if (
      window.confirm(
        `Really delete neighbourhood ${name}? You can't undo this.`,
      )
    ) {
      $app!.deleteNeighbourhoodBoundary(name);
      autosave();
      // TODO Improve perf, don't call this twice
      gj = JSON.parse($app!.toSavefile());
    }
  }

  function renameNeighbourhood(name: string) {
    let newName = window.prompt(`Rename neighbourhood ${name} to what?`, name);
    if (newName) {
      $app!.renameNeighbourhoodBoundary(name, newName);
      autosave();
      gj = JSON.parse($app!.toSavefile());
    }
  }

  function newBoundary() {
    let name = window.prompt("What do you want to name the neighbourhood?");
    if (name) {
      $mode = { mode: "set-boundary", name, existing: null };
    }
  }

  function exportGJ() {
    downloadGeneratedFile($projectName + ".geojson", $app!.toSavefile());
  }

  // TODO Hover on button and highlight on map
</script>

<SplitComponent>
  <div slot="top" style="display: flex; justify-content: space-between;">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title" })}>
            Choose project
          </Link>
        </li>
        <li>Pick neighbourhood</li>
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
        <li>
          <Link on:click={() => ($mode = { mode: "debug-gj" })}>
            Debug route snapper
          </Link>
        </li>
      </ul>
    </nav>
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

    <Link on:click={newBoundary}>Draw a new boundary</Link>
    <ul>
      {#each boundaryNames as name}
        <li style="display: flex; justify-content: space-between;">
          <Link on:click={() => pickNeighbourhood(name)}>
            {name}
          </Link>
          <button class="secondary" on:click={() => renameNeighbourhood(name)}>
            <img
              src={$lightMode ? editLight : editDark}
              alt="Rename neighbourhood"
            />
          </button>
          <button class="secondary" on:click={() => deleteNeighbourhood(name)}>
            <img
              src={$lightMode ? deleteLight : deleteDark}
              alt="Delete neighbourhood"
            />
          </button>
        </li>
      {/each}
    </ul>

    <button on:click={exportGJ}>Export project to GeoJSON</button>
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
    </GeoJSON>
    <ModalFilterLayer />
  </div>
</SplitComponent>
