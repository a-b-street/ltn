<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import { Link, notNull } from "./common";
  import RouteSnapperLayer from "./common/snapper/RouteSnapperLayer.svelte";
  import SnapPolygonControls from "./common/snapper/SnapPolygonControls.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { autosave, app, mode, route_tool } from "./stores";

  export let name: string;
  export let existing: Feature<Polygon> | null;

  if (existing) {
    $route_tool!.editExistingArea(existing);
  } else {
    $route_tool!.startArea();
  }

  // TODO When we click a link and nav away, clear state

  function onFailure() {
    if (existing) {
      $mode = {
        mode: "neighbourhood",
      };
    } else {
      $mode = {
        mode: "network",
      };
    }
    $route_tool!.clearEventListeners();
  }

  $route_tool!.addEventListenerSuccess((feature) => {
    try {
      $app!.setNeighbourhoodBoundary(name, feature);
      autosave();
      $app!.setCurrentNeighbourhood(name);
      $mode = {
        mode: "neighbourhood",
      };
      $route_tool!.clearEventListeners();
    } catch (err) {
      window.alert(
        "Known georust bug hit, sorry. You may need to just refresh the page now.",
      );
      onFailure();
    }
  });
  $route_tool!.addEventListenerFailure(onFailure);
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title" })}>
            Choose project
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "network" })}>
            Pick neighbourhood
          </Link>
        </li>
        {#if existing}
          <li>
            <Link on:click={() => ($mode = { mode: "neighbourhood" })}>
              Editing
            </Link>
          </li>
          <li>Changing neighbourhood boundary</li>
        {:else}
          <li>Creating new neighbourhood boundary</li>
        {/if}
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <h1>Draw your neighbourhood boundary for {name}</h1>

    <div style="display: flex; justify-content: space-between;">
      <Link on:click={() => notNull($route_tool).finish()}>Finish</Link>
      <Link on:click={onFailure}>Cancel</Link>
    </div>

    <p>TODO: maybe move the instructions from the previous screen to here...</p>

    <SnapPolygonControls route_tool={notNull($route_tool)} />
  </div>

  <div slot="map">
    <RouteSnapperLayer />
  </div>
</SplitComponent>
