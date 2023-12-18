<script lang="ts">
  import { MapModel } from "backend";
  import { GeoJSON, LineLayer, Popup } from "svelte-maplibre";
  import { PropertiesTable } from "./common";

  export let model: MapModel;
</script>

<GeoJSON data={JSON.parse(model.render())}>
  <LineLayer
    id="network"
    paint={{
      "line-width": 5,
      "line-color": "black",
    }}
    on:click={(e) => window.open(e.detail.features[0].properties.way, "_blank")}
    hoverCursor="pointer"
  >
    <Popup openOn="hover" let:data>
      <PropertiesTable properties={data.properties} />
    </Popup>
  </LineLayer>
</GeoJSON>
