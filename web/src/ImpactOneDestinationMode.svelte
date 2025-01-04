<script lang="ts">
  import type { Feature, FeatureCollection } from "geojson";
  import { LngLat, type MapMouseEvent } from "maplibre-gl";
  import { FillLayer, GeoJSON, LineLayer, MapEvents } from "svelte-maplibre";
  import { notNull } from "svelte-utils";
  import {
    constructMatchExpression,
    emptyGeojson,
    Popup,
  } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { DotMarker, layerId, Link } from "./common";
  import ModalFilterLayer from "./ModalFilterLayer.svelte";
  import {
    backend,
    mode,
    one_destination,
    route_pt_a,
    route_pt_b,
  } from "./stores";

  function back() {
    $mode = { mode: "neighbourhood" };
  }

  $: perRoadGj = $backend!.impactToOneDestination($one_destination);

  let hovered: Feature | null = null;

  $: routeGj = previewRoutes(hovered);

  function previewRoutes(hovered: Feature | null): FeatureCollection {
    if (!hovered) {
      return emptyGeojson();
    }
    return $backend!.compareRoute(
      new LngLat(hovered.properties!.pt1_x, hovered.properties!.pt1_y),
      $one_destination,
      1.0,
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
          <Link on:click={() => ($mode = { mode: "title", firstLoad: false })}>
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
    <GeoJSON data={notNull($backend).renderNeighbourhood()}>
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

    <DotMarker bind:lngLat={$one_destination} draggable>X</DotMarker>
  </div>
</SplitComponent>
