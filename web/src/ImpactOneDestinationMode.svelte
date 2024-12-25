<script lang="ts">
  import { LngLat, type MapMouseEvent } from "maplibre-gl";
  import type { Feature, FeatureCollection } from "geojson";
  import BackButton from "./BackButton.svelte";
  import { setCellColors } from "./cells";
  import {
    MapEvents,
    FillLayer,
    GeoJSON,
    LineLayer,
    Marker,
  } from "svelte-maplibre";
  import { layerId, Link } from "./common";
  import { notNull } from "svelte-utils";
  import {
    constructMatchExpression,
    emptyGeojson,
    Popup,
  } from "svelte-utils/map";
  import ModalFilterLayer from "./ModalFilterLayer.svelte";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { app, mode, one_destination, route_pt_a, route_pt_b } from "./stores";

  function back() {
    $mode = { mode: "neighbourhood" };
  }

  $: perRoadGj = JSON.parse(
    $app!.impactToOneDestination($one_destination.lng, $one_destination.lat),
  );

  let hovered: Feature | null = null;

  $: routeGj = previewRoutes(hovered);

  function previewRoutes(hovered: Feature | null): FeatureCollection {
    if (!hovered) {
      return emptyGeojson();
    }
    return JSON.parse(
      $app!.compareRoute(
        hovered.properties!.pt1_x,
        hovered.properties!.pt1_y,
        $one_destination.lng,
        $one_destination.lat,
        1.0,
      ),
    );
  }

  function compareRoute(f: Feature) {
    $route_pt_a = new LngLat(f.properties!.pt1_x, f.properties!.pt1_y);
    $route_pt_b = $one_destination;
    $mode = { mode: "route", prevMode: "impact-one-destination" };
  }

  function onRightClick(e: CustomEvent<MapMouseEvent>) {
    // Move the first marker, for convenience
    $one_destination = e.detail.lngLat;
  }
</script>

<MapEvents on:contextmenu={onRightClick} />

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
        bind:hovered
      >
        <Popup openOn="hover" let:props>
          Ratio {(props.distance_after / props.distance_before).toFixed(1)}
        </Popup>
      </LineLayer>
    </GeoJSON>

    <GeoJSON data={routeGj}>
      <LineLayer
        {...layerId("compare-route")}
        paint={{
          "line-width": 10,
          "line-color": constructMatchExpression(
            ["get", "kind"],
            {
              before: "red",
              after: "blue",
            },
            "red",
          ),
        }}
      />
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
