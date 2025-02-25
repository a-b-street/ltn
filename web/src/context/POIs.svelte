<script lang="ts">
  import { CircleLayer, GeoJSON } from "svelte-maplibre";
  import { QualitativeLegend } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { HelpButton, layerId } from "../common";
  import { assetUrl } from "../stores";

  let show = false;

  let colors = {
    Schools: "#7fc97f",
    GPs: "#beaed4",
    Hospitals: "#fdc086",
  };
</script>

<button class="secondary" data-expanded={show} on:click={() => (show = !show)}>POIs</button>
{#if show}
  <div>
    <QualitativeLegend {colors} horiz />
    <HelpButton>
      <p>
        See Scottish data sources for <a
          href="https://www.data.gov.uk/dataset/9a6f9d86-9698-4a5d-a2c8-89f3b212c52c/scottish-school-roll-and-locations"
          target="_blank"
        >
          schools
        </a>
        ,
        <a
          href="https://data.spatialhub.scot/dataset/gp_practices-is"
          target="_blank"
        >
          GP practices
        </a>
        , and
        <a
          href="https://data.spatialhub.scot/dataset/nhs_hospitals-is"
          target="_blank"
        >
          hospitals
        </a>
        .
      </p>
    </HelpButton>
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
