<script lang="ts">
  import { LineLayer, VectorTileSource } from "svelte-maplibre";
  import { QualitativeLegend, SequentialLegend } from "svelte-utils";
  import { constructMatchExpression } from "svelte-utils/map";
  import { HelpButton, layerId, roadLineWidth } from "../common";
  import { assetUrl } from "../stores";

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

<button class="secondary" on:click={() => (showTraffic = !showTraffic)}>
  Traffic
</button>
{#if showTraffic}
  <div>
    <SequentialLegend colorScale={traffic.colorScale} limits={traffic.limits} />
    <HelpButton>
      <a
        href="https://nptscot.github.io/manual/#infrastructureandtraffic"
        target="_blank"
      >
        Data from NPT
      </a>
    </HelpButton>
  </div>
{/if}

<button class="secondary" on:click={() => (showLos = !showLos)}>
  Level of Service
</button>
{#if showLos}
  <div>
    <QualitativeLegend colors={levelOfServiceColors} horiz />
    <HelpButton>
      <a
        href="https://nptscot.github.io/manual/#infrastructureandtraffic"
        target="_blank"
      >
        Data from NPT
      </a>
    </HelpButton>
  </div>
{/if}

<button
  class="secondary"
  on:click={() => (showExistingInfra = !showExistingInfra)}
>
  Existing cycle infrastructure
</button>
{#if showExistingInfra}
  <div>
    <QualitativeLegend colors={infraTypeColors} horiz />
    <HelpButton>
      <a
        href="https://nptscot.github.io/manual/#infrastructureandtraffic"
        target="_blank"
      >
        Data from NPT
      </a>
    </HelpButton>
  </div>
{/if}

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
