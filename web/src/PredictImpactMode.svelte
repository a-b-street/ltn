<script lang="ts">
  import { run } from 'svelte/legacy';

  import type { Feature } from "geojson";
  import { FillLayer, GeoJSON, LineLayer, Popup } from "svelte-maplibre";
  import { SequentialLegend } from "svelte-utils";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import {
    HelpButton,
    layerId,
    Loading,
    ModeLink,
    pageTitle,
    refreshLoadingScreen,
  } from "./common";
  import { ModalFilterLayer } from "./layers";
  import {
    appFocus,
    backend,
    fastSample,
    minImpactCount,
    mode,
  } from "./stores";
  import type { Impact } from "./wasm";

  interface Props {
    prevMode: "pick-neighbourhood" | "neighbourhood";
  }

  let { prevMode }: Props = $props();

  // Based partly on https://colorbrewer2.org/#type=diverging&scheme=RdYlGn&n=5
  // The middle color white doesn't matter; the source data will filter out unchanged roads
  let divergingScale = ["#1a9641", "#a6d96a", "white", "#fdae61", "#d7191c"];

  let loading = $state("");
  let impactGj: Impact = $state({
    type: "FeatureCollection" as const,
    features: [],
    max_count: 1,
  });
  let neighbourhoods = $backend!.toSavefile();

  let minRoadWidth = 3;
  let maxRoadWidth = 10;

  function pickRoad(f: Feature) {
    $mode = { mode: "impact-detail", road: f, prevPrevMode: prevMode };
  }

  async function recalculate(fastSample: boolean) {
    loading = "Calculating impact";
    await refreshLoadingScreen();
    impactGj = $backend!.predictImpact(fastSample);
    loading = "";
  }

  let fastSampleRadio = $state($fastSample ? "fast" : "accurate");
  function updateFastSample() {
    $fastSample = fastSampleRadio == "fast";
  }
  run(() => {
    recalculate($fastSample);
  });
</script>

<Loading {loading} />

<SplitComponent>
  {#snippet top()}
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
  {/snippet}

  {#snippet left()}
    <BackButton mode={{ mode: prevMode }} />

    <p>
      This mode estimates the impact of all your changes on traffic around the
      entire area. It's based on many assumptions and must be interpreted very
      carefully. <ModeLink mode={{ mode: "debug-demand" }}>
        Explore the origin/destination demand data used
      </ModeLink>
    </p>

    <div style="display: flex">
      {#if $appFocus == "cnt"}
        <span>About the 2011 home to work data</span>
        <HelpButton>
          <p>
            This region uses preprocessed census data from the <a
              href="https://nptscot.github.io/data/"
              target="_blank">NPT project</a
            >. The data is very outdated. Trips to work are just one purpose,
            representing a small percent of all trips and with strong spatial
            biases.
          </p>
        </HelpButton>
      {:else if $appFocus == "england"}
        <span>About the 2021 home to work data</span>
        <HelpButton>
          <p>
            This region uses the <a
              href="https://www.nomisweb.co.uk/sources/census_2021_od"
              target="_blank">ODWP01EW</a
            >
            census dataset, with trips from home to work. It was taken during COVID-19,
            so
            <a
              href="https://www.ons.gov.uk/peoplepopulationandcommunity/populationandmigration/populationestimates/methodologies/userguidetocensus2021origindestinationdataenglandandwales"
              target="_blank">workplace travel patterns have major caveats</a
            >. Trips to work are just one purpose, representing a small percent
            of all trips and with strong spatial biases. This dataset does not
            distinguish by trip mode, so trips could be made by driving, public
            transit, cycling, walking, etc.
          </p>
        </HelpButton>
      {:else}
        <b>About the fake origin/destination data</b>
        <HelpButton>
          <p>
            This region has no available origin/destination data about travel
            patterns, so a <b>completely random</b> small set of trips are
            modelled. Contact Dustin at
            <a href="mailto:dabreegster@gmail.com">dabreegster@gmail.com</a> to set
            up real data in your region.
          </p>
        </HelpButton>
      {/if}
    </div>

    <hr />

    <fieldset>
      <label>
        <input
          type="radio"
          value="fast"
          bind:group={fastSampleRadio}
          onchange={updateFastSample}
        />
        Calculate quickly
      </label>
      <label>
        <input
          type="radio"
          value="accurate"
          bind:group={fastSampleRadio}
          onchange={updateFastSample}
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
  {/snippet}

  {#snippet main()}
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
        onclick={(e) => pickRoad(e.features[0])}
      >
        <Popup openOn="hover">
          {#snippet children({ data })}
            {@const props = data!.properties!}
            <p>
              {props.before.toLocaleString()} before, {props.after.toLocaleString()}
              after
            </p>
            <p>{Math.round((100 * props.after) / props.before)}%</p>
          {/snippet}
        </Popup>
      </LineLayer>
    </GeoJSON>

    <ModalFilterLayer interactive={false} />
  {/snippet}
</SplitComponent>
