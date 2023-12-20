<script lang="ts">
  import { LTN } from "backend";
  import { GeoJSON, LineLayer, Popup } from "svelte-maplibre";
  import { PropertiesTable } from "./common";

  export let app: LTN;
</script>

<GeoJSON data={JSON.parse(app.render())}>
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
