<script lang="ts">
  import { LngLat } from "maplibre-gl";
  import type { Feature } from "geojson";
  import BackButton from "./BackButton.svelte";
  import { setCellColors } from "./cells";
  import { FillLayer, GeoJSON, LineLayer, Marker } from "svelte-maplibre";
  import { layerId, Popup, Link } from "./common";
  import { notNull } from "svelte-utils";
  import ModalFilterLayer from "./ModalFilterLayer.svelte";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { app, mode, one_destination, route_pt_a, route_pt_b } from "./stores";

  function back() {
    $mode = { mode: "neighbourhood" };
  }

  $: perRoadGj = JSON.parse(
    $app!.impactToOneDestination($one_destination.lng, $one_destination.lat),
  );

  function compareRoute(f: Feature) {
    $route_pt_a = new LngLat(f.properties!.pt1_x, f.properties!.pt1_y);
    $route_pt_b = $one_destination;
    $mode = { mode: "route", prevMode: "impact-one-destination" };
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title" })}>
            Choose project
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "network" })}>
            Pick neighbourhood
          </Link>
        </li>
        <li>
          <Link on:click={back}>Editing</Link>
        </li>
        <li>Impact routing to one destination</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton on:click={back} />

    <p>
      This shows the change in driving time to one destination from everywhere
      within the neighbourhood. Drag the pin aroun to change that destination.
    </p>
    <p>TODO: It's just distance right now, not time</p>
    <p>Highest ratio is {perRoadGj.highest_ratio.toFixed(1)}</p>
  </div>

  <div slot="map">
    <GeoJSON
      data={setCellColors(JSON.parse(notNull($app).renderNeighbourhood()))}
    >
      <FillLayer
        {...layerId("cells")}
        filter={["==", ["get", "kind"], "cell"]}
        paint={{
          "fill-color": ["get", "color"],
          "fill-opacity": 0.6,
        }}
      />
    </GeoJSON>

    <GeoJSON data={perRoadGj} generateId>
      <LineLayer
        {...layerId("interior-roads")}
        paint={{
          "line-color": [
            "interpolate-hcl",
            ["linear"],
            ["/", ["get", "distance_after"], ["get", "distance_before"]],
            1,
            "white",
            perRoadGj.highest_ratio,
            "red",
          ],
          "line-width": 5,
        }}
        manageHoverState
        on:click={(e) => compareRoute(e.detail.features[0])}
      >
        <Popup openOn="hover" let:props>
          Ratio {(props.distance_after / props.distance_before).toFixed(1)}
        </Popup>
      </LineLayer>
    </GeoJSON>

    <ModalFilterLayer />

    <Marker bind:lngLat={$one_destination} draggable>
      <span class="dot">X</span>
    </Marker>
  </div>
</SplitComponent>

<style>
  .dot {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: flex;
    justify-content: center;
    align-items: center;

    background-color: grey;
    font-weight: bold;
  }
</style>
