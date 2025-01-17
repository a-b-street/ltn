<script lang="ts">
  import type { Feature } from "geojson";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { layerId, Link } from "./common";
  import { ModalFilterLayer } from "./layers";
  import { backend, mode } from "./stores";

  // Based partly on https://colorbrewer2.org/#type=diverging&scheme=RdYlGn&n=5
  // The middle color white doesn't matter; the source data will filter out unchanged roads
  let divergingScale = ["#1a9641", "#a6d96a", "white", "#fdae61", "#d7191c"];

  let data = $backend!.predictImpact();

  let minRoadWidth = 3;
  let maxRoadWidht = 10;

  function pickRoad(f: Feature) {
    $mode = { mode: "impact-detail", road: f };
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title", firstLoad: false })}>
            Choose project
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "network" })}>
            Pick neighbourhood
          </Link>
        </li>
        <li>Predict impact mode</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton on:click={() => ($mode = { mode: "network" })} />

    <p>
      This mode estimates the impact of all your changes on traffic around the
      entire area. It's based on many assumptions and must be interpreted very
      carefully.
    </p>
    <p>
      Red roads have increased traffic, and green roads have decreased. If
      hovering on a road doesn't show anything, there was no change there. Click
      a road to see example routes through it that've changed.
    </p>
    <p>
      Thicker roads have more traffic after edits, relative to the max count for
      any road: {data.max_count.toLocaleString()}
    </p>
  </div>

  <div slot="map">
    <GeoJSON {data} generateId>
      <LineLayer
        {...layerId("predict-impact")}
        paint={{
          "line-width": [
            "interpolate",
            ["linear"],
            ["get", "after"],
            0,
            minRoadWidth,
            data.max_count,
            maxRoadWidht,
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

    <ModalFilterLayer />
  </div>
</SplitComponent>
