<script lang="ts">
  import { GeoJSON, LineLayer, Popup } from "svelte-maplibre";
  import { PropertiesTable } from "./common";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode } from "./stores";
</script>

<SplitComponent>
  <div slot="sidebar">
    <div>
      <button
        on:click={() => ($mode = { mode: "set-boundary", existing: null })}
        >Set boundary</button
      >
    </div>
  </div>

  <div slot="map">
    <GeoJSON data={JSON.parse($app.render())}>
      <LineLayer
        id="network"
        paint={{
          "line-width": 5,
          "line-color": "black",
        }}
        on:click={(e) =>
          window.open(e.detail.features[0].properties.way, "_blank")}
        hoverCursor="pointer"
      >
        <Popup openOn="hover" let:data>
          <PropertiesTable properties={data.properties} />
        </Popup>
      </LineLayer>
    </GeoJSON>
  </div>
</SplitComponent>
