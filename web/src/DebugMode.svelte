<script lang="ts">
  import { CircleLayer, GeoJSON, LineLayer } from "svelte-maplibre";
  import { layerId, notNull, PropertiesTable, Popup } from "./common";
  import RenderNeighbourhood from "./RenderNeighbourhood.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode } from "./stores";
</script>

<SplitComponent>
  <div slot="sidebar">
    <h1>Debug mode</h1>

    <button on:click={() => ($mode = { mode: "neighbourhood" })}
      >Back to editing</button
    >
  </div>

  <div slot="map">
    <RenderNeighbourhood
      gjInput={JSON.parse(notNull($app).renderNeighbourhood())}
      interactive
      onClickLine={(f) => window.open(notNull(f.properties).way, "_blank")}
    >
      <div slot="line-popup">
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </div>
      <svelte:fragment slot="more-layers">
        <CircleLayer
          {...layerId("debug-borders")}
          filter={["==", ["get", "kind"], "border_intersection"]}
          paint={{
            "circle-radius": 15,
            "circle-color": "green",
          }}
        >
          <Popup openOn="hover" let:props>
            <PropertiesTable properties={props} />
          </Popup>
        </CircleLayer>
        <LineLayer
          {...layerId("debug-crosses")}
          filter={["==", ["get", "kind"], "crosses"]}
          paint={{
            "line-width": 5,
            "line-color": "blue",
          }}
        >
          <Popup openOn="hover" let:props>
            <PropertiesTable properties={props} />
          </Popup>
        </LineLayer>
      </svelte:fragment>
    </RenderNeighbourhood>

    <GeoJSON data={JSON.parse(notNull($app).renderModalFilters())} generateId>
      <CircleLayer
        {...layerId("debug-filters")}
        paint={{
          "circle-radius": 15,
          "circle-color": "black",
        }}
      >
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </CircleLayer>
    </GeoJSON>
  </div>
</SplitComponent>
