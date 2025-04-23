<script lang="ts">
  import { LineLayer, VectorTileSource } from "svelte-maplibre";
  import { constructMatchExpression } from "svelte-utils/map";
  import {
    ContextLayerButton,
    layerId,
    QualitativeLegend,
    roadLineWidth,
    SequentialLegend,
  } from "../common";
  import { assetUrl } from "../stores";
  import RouteNetwork from "./RouteNetwork.svelte";

  // The NPT project bundles together a few layers into one pmtiles file, all
  // related to the Cycling By Design guidance
  // (https://www.transport.gov.scot/media/50323/cycling-by-design-update-2019-final-document-15-september-2021-1.pdf).

  let showTraffic = false;
  let showLos = false;
  let showExistingInfra = false;

  let traffic = {
    colorScale: ["#27918d", "#ffaa33", "#440154"],
    limits: [0, 2000, 4000, 10000],
  };

  let levelOfServiceColors = {
    High: "mediumseagreen",
    Medium: "orange",
    Low: "red",
    "Should not be used": "brown",
  };

  let infraTypeColors = {
    "Segregated Track (wide)": "#054d05",
    "Off Road Cycleway": "#3a9120",
    "Segregated Track (narrow)": "#87d668",
    "Shared Footway": "#ffbf00",
    "Painted Cycle Lane": "#7faedd",
  };
</script>

<ContextLayerButton
  bind:show={showExistingInfra}
  label="Existing cycling infrastructure"
>
  <div slot="legend">
    <QualitativeLegend labelColors={infraTypeColors} />
  </div>

  <p slot="help">
    <a
      href="https://nptscot.github.io/manual/#infrastructureandtraffic"
      target="_blank"
    >
      Data from NPT
    </a>
  </p>
</ContextLayerButton>

<ContextLayerButton bind:show={showLos} label="Cycling safety Level of Service">
  <div slot="legend">
    <SequentialLegend
      colorScale={Object.values(levelOfServiceColors)}
      labels={{ buckets: Object.keys(levelOfServiceColors) }}
      fullWidthBucketLegend
    />
  </div>

  <p slot="help">
    <a
      href="https://nptscot.github.io/manual/#infrastructureandtraffic"
      target="_blank"
    >
      Data from NPT
    </a>
  </p>
</ContextLayerButton>

<RouteNetwork />

<div class="layer-group">Other</div>

<ContextLayerButton bind:show={showTraffic} label="Estimated traffic">
  <div slot="legend">
    <SequentialLegend
      colorScale={traffic.colorScale}
      labels={{ limits: traffic.limits }}
    />
  </div>

  <p slot="help">
    <a
      href="https://nptscot.github.io/manual/#infrastructureandtraffic"
      target="_blank"
    >
      Data from NPT
    </a>
  </p>
</ContextLayerButton>

<VectorTileSource url={`pmtiles://${assetUrl("cnt_layers/cbd.pmtiles")}`}>
  <LineLayer
    {...layerId("context-traffic")}
    sourceLayer="cbd_layer"
    filter={["has", "Traffic volume category"]}
    paint={{
      "line-color": constructMatchExpression(
        ["get", "Traffic volume category"],
        {
          "0 to 1999": traffic.colorScale[0],
          "2000 to 3999": traffic.colorScale[1],
          "4000+": traffic.colorScale[2],
        },
        "cyan",
      ),
      "line-width": roadLineWidth(0),
    }}
    layout={{
      visibility: showTraffic ? "visible" : "none",
    }}
  />

  <LineLayer
    {...layerId("context-los")}
    sourceLayer="cbd_layer"
    paint={{
      "line-color": constructMatchExpression(
        ["get", "Level of Service"],
        levelOfServiceColors,
        "cyan",
      ),
      "line-width": roadLineWidth(1),
    }}
    layout={{
      visibility: showLos ? "visible" : "none",
    }}
  />

  <LineLayer
    {...layerId("context-existing-infra")}
    sourceLayer="cbd_layer"
    filter={["!=", ["get", "Infrastructure type"], "Mixed Traffic Street"]}
    paint={{
      "line-color": constructMatchExpression(
        ["get", "Infrastructure type"],
        infraTypeColors,
        "cyan",
      ),
      "line-width": roadLineWidth(1),
    }}
    layout={{
      visibility: showExistingInfra ? "visible" : "none",
    }}
  />
</VectorTileSource>
