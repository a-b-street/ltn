<script lang="ts">
  import {
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
  } from "svelte-maplibre";
  import { emptyGeojson, Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { layerId, Link } from "./common";
  import { backend, mode } from "./stores";

  let gj = emptyGeojson();
  try {
    gj = $backend!.getDemandModel();
  } catch (err) {
    window.alert("No demand model for this area");
    $mode = { mode: "pick-neighbourhood" };
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
          <Link on:click={() => ($mode = { mode: "pick-neighbourhood" })}>
            Pick neighbourhood
          </Link>
        </li>
        <li>Debug demand model</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton on:click={() => ($mode = { mode: "pick-neighbourhood" })} />

    <p>{gj.features.length.toLocaleString()} zones</p>
  </div>

  <div slot="map">
    <GeoJSON data={gj} promoteId="id">
      <FillLayer
        {...layerId("debug-demand-fill")}
        paint={{
          "fill-color": "grey",
          "fill-opacity": hoverStateFilter(0.5, 0.1),
        }}
        manageHoverState
      >
        <Popup openOn="hover" let:props>{props.id}</Popup>
      </FillLayer>

      <LineLayer
        {...layerId("debug-demand-outline")}
        paint={{
          "line-width": 2,
          "line-color": "black",
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
