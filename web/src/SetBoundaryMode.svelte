<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import { notNull } from "./common";
  import RouteSnapperLayer from "./common/snapper/RouteSnapperLayer.svelte";
  import SnapPolygonControls from "./common/snapper/SnapPolygonControls.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode, route_tool } from "./stores";

  export let name: string;
  export let existing: Feature<Polygon> | null;

  if (existing) {
    $route_tool!.editExistingArea(existing);
  } else {
    $route_tool!.startArea();
  }

  function onFailure() {
    $mode = {
      mode: "network",
    };
    $route_tool!.clearEventListeners();
  }

  $route_tool!.addEventListenerSuccess((feature) => {
    try {
      $app!.setNeighbourhoodBoundary(name, feature);
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
  <div slot="sidebar">
    <h1>Draw your neighbourhood boundary for {name}</h1>

    <div style="display: flex; justify-content: space-between;">
      <button on:click={() => notNull($route_tool).finish()}>Finish</button>
      <button class="secondary" on:click={onFailure}>Cancel</button>
    </div>

    <p>TODO: maybe move the instructions from the previous screen to here...</p>

    <SnapPolygonControls route_tool={notNull($route_tool)} />
  </div>

  <div slot="map">
    <RouteSnapperLayer />
  </div>
</SplitComponent>
