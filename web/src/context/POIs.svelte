<script lang="ts">
  import { CircleLayer, ControlButton, GeoJSON } from "svelte-maplibre";
  import { QualitativeLegend } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { layerId } from "../common";
  import { assetUrl } from "../stores";

  let show = false;

  let colors = {
    Schools: "#7fc97f",
    GPs: "#beaed4",
    Hospitals: "#fdc086",
  };
</script>

<ControlButton on:click={() => (show = !show)}>POIs</ControlButton>
{#if show}
  <div>
    <QualitativeLegend {colors} horiz />
  </div>
{/if}

<GeoJSON data={assetUrl("cnt_layers/gp_practices.geojson")} generateId>
  <CircleLayer
    {...layerId("context-gp-practices")}
    paint={{
      "circle-color": colors.GPs,
      "circle-radius": 10,
      "circle-stroke-color": "black",
      "circle-stroke-width": 1,
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
  >
    <Popup openOn="hover" let:props>{props.name}</Popup>
  </CircleLayer>
</GeoJSON>

<GeoJSON data={assetUrl("cnt_layers/hospitals.geojson")} generateId>
  <CircleLayer
    {...layerId("context-hospitals")}
    paint={{
      "circle-color": colors.Hospitals,
      "circle-radius": 10,
      "circle-stroke-color": "black",
      "circle-stroke-width": 1,
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
  >
    <Popup openOn="hover" let:props>{props.name}</Popup>
  </CircleLayer>
</GeoJSON>

<GeoJSON data={assetUrl("cnt_layers/schools.geojson")} generateId>
  <CircleLayer
    {...layerId("context-schools")}
    paint={{
      "circle-color": colors.Schools,
      "circle-radius": 10,
      "circle-stroke-color": "black",
      "circle-stroke-width": 1,
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
  >
    <Popup openOn="hover" let:props>
      {props.name} with {props.pupils} pupils
    </Popup>
  </CircleLayer>
</GeoJSON>
