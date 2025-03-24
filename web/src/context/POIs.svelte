<script lang="ts">
  import { CircleLayer, GeoJSON } from "svelte-maplibre";
  import { constructMatchExpression, Popup } from "svelte-utils/map";
  import { ContextLayerButton, layerId, QualitativeLegend } from "../common";
  import { backend } from "../stores";

  let show = false;

  // https://colorbrewer2.org/#type=qualitative&scheme=Accent&n=5
  let labelColors = {
    School: "#7fc97f",
    GP: "#beaed4",
    Hospital: "#fdc086",
    Grocery: "#386cb0",
    CommunityCenter: "#ffff99",
  };
</script>

<ContextLayerButton label="POIs" bind:show>
  <div slot="legend">
    <QualitativeLegend {labelColors} swatchClass="circle" />
  </div>

  <p slot="help">
    See Scottish data sources for
    <a
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
    . Other data is from
    <a href="https://www.openstreetmap.org/about" target="_blank">
      OpenStreetMap
    </a>
    .
  </p>
</ContextLayerButton>

{#if $backend}
  <GeoJSON data={$backend.getPOIs()} generateId>
    <CircleLayer
      {...layerId("context-pois")}
      paint={{
        "circle-color": constructMatchExpression(
          ["get", "kind"],
          labelColors,
          "black",
        ),
        "circle-radius": 10,
        "circle-stroke-color": "black",
        "circle-stroke-width": 1,
      }}
      layout={{
        visibility: show ? "visible" : "none",
      }}
    >
      <Popup openOn="hover" let:props>
        {props.name || `unnamed ${props.kind}`}
      </Popup>
    </CircleLayer>
  </GeoJSON>
{/if}
