<script lang="ts">
  import { GeoJSON, LineLayer, type LayerClickInfo } from "svelte-maplibre";
  import { QualitativeLegend } from "svelte-utils";
  import { constructMatchExpression, emptyGeojson } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { layerId, ModeLink, pageTitle, PrevNext } from "./common";
  import { backend, mode } from "./stores";

  let colors = {
    low: "#27918d",
    medium: "#ffaa33",
    high: "#440154",
  };

  let widths = {
    low: 2,
    medium: 6,
    high: 12,
  };
</script>

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
        <li>{pageTitle($mode.mode)}</li>
      </ul>
    </nav>
  {/snippet}

  {#snippet left()}
    <BackButton mode={{ mode: "pick-neighbourhood" }} />

    <QualitativeLegend labelColors={colors} itemsPerRow={1} />
  {/snippet}

  {#snippet main()}
    <GeoJSON data={$backend!.getAllTrafficPredictions()} generateId>
      <LineLayer
        {...layerId("debug-traffic")}
        paint={{
          "line-width": constructMatchExpression(
            ["get", "traffic"],
            widths,
            20,
          ),
          "line-color": constructMatchExpression(
            ["get", "traffic"],
            colors,
            "red",
          ),
        }}
      />
    </GeoJSON>
  {/snippet}
</SplitComponent>
