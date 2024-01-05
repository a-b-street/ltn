<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import { RouteTool } from "./common/route_tool";
  import RouteSnapperLayer from "./common/RouteSnapperLayer.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode } from "./stores";

  export let route_tool: RouteTool;
  export let existing: Feature<Polygon> | null;

  if (existing) {
    route_tool.editExistingArea(existing);
  } else {
    route_tool.startArea();
  }

  route_tool.addEventListenerSuccess((feature) => {
    try {
      $app!.setNeighbourhood(feature);
      $mode = {
        mode: "neighbourhood",
      };
      route_tool.clearEventListeners();
    } catch (err) {
      window.alert("Known georust bug hit, sorry");
      $mode = {
        mode: "network",
      };
      route_tool.clearEventListeners();
    }
  });
  route_tool.addEventListenerFailure(() => {
    $mode = {
      mode: "network",
    };
    route_tool.clearEventListeners();
  });
</script>

<SplitComponent>
  <div slot="sidebar">
    <p>Draw the boundary...</p>
  </div>

  <div slot="map">
    <RouteSnapperLayer />
  </div>
</SplitComponent>
