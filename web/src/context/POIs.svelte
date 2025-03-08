<script lang="ts">
  import { CircleLayer, GeoJSON } from "svelte-maplibre";
  import { notNull, QualitativeLegend } from "svelte-utils";
  import { constructMatchExpression, Popup } from "svelte-utils/map";
  import { layerId } from "../common";
  import ContextLayerButton from "../common/ContextLayerButton.svelte";
  import { assetUrl, backend } from "../stores";

  let show = false;

  // https://colorbrewer2.org/#type=qualitative&scheme=Accent&n=5
  let colors = {
    School: "#7fc97f",
    GP: "#beaed4",
    Hospital: "#fdc086",
    Grocery: "#386cb0",
    CommunityCenter: "#ffff99",
  };
</script>

<ContextLayerButton label="POIs" bind:show>
  <div slot="legend">
    <QualitativeLegend {colors} horiz />
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

<GeoJSON data={assetUrl("cnt_layers/gp_practices.geojson")} generateId>
  <CircleLayer
    {...layerId("context-gp-practices")}
    paint={{
      "circle-color": colors.GP,
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
      "circle-color": colors.Hospital,
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
      "circle-color": colors.School,
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

<GeoJSON data={notNull($backend).getPOIs()} generateId>
  <CircleLayer
    {...layerId("context-pois")}
    paint={{
      "circle-color": constructMatchExpression(
        ["get", "kind"],
        colors,
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
