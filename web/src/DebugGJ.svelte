<script lang="ts">
  import {
    hoverStateFilter,
    CircleLayer,
    GeoJSON,
    LineLayer,
  } from "svelte-maplibre";
  import { Popup, notNull, PropertiesTable } from "./common";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode } from "./stores";

  // TODO This should just be a standalone tool
</script>

<SplitComponent>
  <div slot="sidebar">
    <h1>Debug GJ</h1>

    <div>
      <button on:click={() => ($mode = { mode: "network" })}>Back</button>
    </div>
  </div>

  <div slot="map">
    <GeoJSON data={JSON.parse(notNull($app).toRouteSnapperGj())} generateId>
      <LineLayer
        manageHoverState
        paint={{
          "line-width": 8,
          "line-color": "black",
          "line-opacity": hoverStateFilter(0.5, 1.0),
        }}
      >
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </LineLayer>
      <CircleLayer
        manageHoverState
        paint={{
          "circle-radius": 10,
          "circle-color": "black",
          "circle-opacity": hoverStateFilter(0.5, 1.0),
        }}
      >
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </CircleLayer>
    </GeoJSON>
  </div>
</SplitComponent>
