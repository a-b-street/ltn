<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import { RouteTool } from "./common/route_tool";
  import RouteSnapperLayer from "./common/RouteSnapperLayer.svelte";
  import SnapPolygonControls from "./common/SnapPolygonControls.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode } from "./stores";

  export let route_tool: RouteTool;
  export let existing: Feature<Polygon> | null;

  if (existing) {
    route_tool.editExistingArea(existing);
  } else {
    route_tool.startArea();
  }

  function onFailure() {
    $mode = {
      mode: "network",
    };
    route_tool.clearEventListeners();
  }

  route_tool.addEventListenerSuccess((feature) => {
    try {
      $app!.setNeighbourhoodBoundary("fixed", feature);
      $app!.setCurrentNeighbourhood("fixed");
      $mode = {
        mode: "neighbourhood",
      };
      route_tool.clearEventListeners();
    } catch (err) {
      window.alert(
        "Known georust bug hit, sorry. You may need to just refresh the page now."
      );
      onFailure();
    }
  });
  route_tool.addEventListenerFailure(onFailure);
</script>

<SplitComponent>
  <div slot="sidebar">
    <h1>Draw your neighbourhood boundary</h1>
    <p>TODO: maybe move the instructions from the previous screen to here...</p>

    <SnapPolygonControls {route_tool} />

    <div>
      <button on:click={() => route_tool.finish()}>Finish</button>
      <button on:click={onFailure}>Cancel</button>
    </div>
  </div>

  <div slot="map">
    <RouteSnapperLayer />
  </div>
</SplitComponent>
