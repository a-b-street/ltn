<script lang="ts">
  import type { Feature } from "geojson";
  import { FillLayer, GeoJSON, LineLayer } from "svelte-maplibre";
  import { SequentialLegend } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { layerId, Loading, ModeLink, pageTitle } from "./common";
  import { ModalFilterLayer } from "./layers";
  import { backend, fastSample, minImpactCount, mode } from "./stores";
  import type { Impact } from "./wasm";

  export let prevMode: "pick-neighbourhood" | "neighbourhood";

  // Based partly on https://colorbrewer2.org/#type=diverging&scheme=RdYlGn&n=5
  // The middle color white doesn't matter; the source data will filter out unchanged roads
  let divergingScale = ["#1a9641", "#a6d96a", "white", "#fdae61", "#d7191c"];

  let loading = "";
  let impactGj: Impact = {
    type: "FeatureCollection" as const,
    features: [],
    max_count: 1,
  };
  $: recalculate($fastSample);
  let neighbourhoods = $backend!.getAllNeighbourhoods();

  let minRoadWidth = 3;
  let maxRoadWidth = 10;

  function pickRoad(f: Feature) {
    $mode = { mode: "impact-detail", road: f, prevPrevMode: prevMode };
  }

  async function recalculate(fastSample: boolean) {
    loading = "Calculating impact";
    // Render the loading screen before starting the calculation. Unsure why Svelte tick() or one frame doesn't work.
    await new Promise((resolve) => {
      requestAnimationFrame(() => {
        requestAnimationFrame(resolve);
      });
    });

    impactGj = $backend!.predictImpact(fastSample);
    loading = "";
  }

  let fastSampleRadio = $fastSample ? "fast" : "accurate";
  function updateFastSample() {
    $fastSample = fastSampleRadio == "fast";
  }
</script>

<Loading {loading} />

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <ModeLink mode={{ mode: "title" }} />
        </li>
        <li>
          <ModeLink mode={{ mode: "pick-neighbourhood" }} />
        </li>
        {#if prevMode == "neighbourhood"}
          <li>
            <ModeLink mode={{ mode: "neighbourhood" }} />
          </li>
        {/if}
        <li>{pageTitle($mode.mode)}</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton mode={{ mode: prevMode }} />

    <p>
      This mode estimates the impact of all your changes on traffic around the
      entire area. It's based on many assumptions and must be interpreted very
      carefully. <ModeLink mode={{ mode: "debug-demand" }}>
        Explore the origin/destination demand data used
      </ModeLink>
    </p>

    <fieldset>
      <label>
        <input
          type="radio"
          value="fast"
          bind:group={fastSampleRadio}
          on:change={updateFastSample}
        />
        Calculate quickly
      </label>
      <label>
        <input
          type="radio"
          value="accurate"
          bind:group={fastSampleRadio}
          on:change={updateFastSample}
        />
        Calculate more accurately
      </label>
    </fieldset>

    <p>
      Red roads have increased traffic, and green roads have decreased. Thicker
      roads have more traffic after edits. If hovering on a road doesn't show
      anything, there was no change there. Click a road to see example routes
      through it that've changed.
    </p>
    <label>
      Only show roads with at least this many daily trips before or after
      <input type="number" min={0} bind:value={$minImpactCount} />
    </label>

    <SequentialLegend
      colorScale={divergingScale}
      labels={{ limits: ["0%", "50%", "same", "150%", "200%"] }}
    />
  </div>

  <div slot="map">
    <GeoJSON data={neighbourhoods}>
      <FillLayer
        {...layerId("neighbourhood-boundaries", false)}
        filter={["==", ["get", "kind"], "boundary"]}
        paint={{
          "fill-color": "grey",
          "fill-opacity": 0.5,
        }}
      />
    </GeoJSON>

    <GeoJSON data={impactGj} generateId>
      <LineLayer
        {...layerId("predict-impact-outline")}
        filter={[
          ">=",
          ["max", ["get", "before"], ["get", "after"]],
          $minImpactCount,
        ]}
        paint={{
          "line-width": [
            "interpolate",
            ["linear"],
            ["get", "after"],
            0,
            1.5 * minRoadWidth,
            impactGj.max_count,
            1.5 * maxRoadWidth,
          ],
          "line-color": "black",
        }}
      />

      <LineLayer
        {...layerId("predict-impact")}
        filter={[
          ">=",
          ["max", ["get", "before"], ["get", "after"]],
          $minImpactCount,
        ]}
        paint={{
          "line-width": [
            "interpolate",
            ["linear"],
            ["get", "after"],
            0,
            minRoadWidth,
            impactGj.max_count,
            maxRoadWidth,
          ],
          "line-color": [
            "let",
            "ratio",
            ["*", 100, ["/", ["get", "after"], ["get", "before"]]],
            [
              "interpolate-hcl",
              ["linear"],
              ["var", "ratio"],
              0,
              divergingScale[0],
              50,
              divergingScale[1],
              100,
              divergingScale[2],
              150,
              divergingScale[3],
              200,
              divergingScale[4],
            ],
          ],
        }}
        manageHoverState
        hoverCursor="pointer"
        on:click={(e) => pickRoad(e.detail.features[0])}
      >
        <Popup openOn="hover" let:props>
          <p>
            {props.before.toLocaleString()} before, {props.after.toLocaleString()}
            after
          </p>
          <p>{Math.round((100 * props.after) / props.before)}%</p>
        </Popup>
      </LineLayer>
    </GeoJSON>

    <ModalFilterLayer interactive={false} />
  </div>
</SplitComponent>
