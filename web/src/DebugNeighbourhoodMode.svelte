<script lang="ts">
  import type { FeatureCollection } from "geojson";
  import {
    CircleLayer,
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { notNull, PropertiesTable } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import {
    layerId,
    mapMetersToPixels,
    ModeLink,
    pageTitle,
    PrevNext,
    Style,
  } from "./common";
  import type { IntersectionFeature } from "./common/Intersection";
  import {
    CellLayer,
    HighlightBoundaryLayer,
    NeighbourhoodRoadLayer,
    OneWayLayer,
    RenderNeighbourhood,
  } from "./layers";
  import ModalFilterLayer from "./layers/ModalFilterLayer.svelte";
  import { backend, mode } from "./stores";

  let intersection: DebugIntersection | null = null;
  type DebugIntersection = {
    feature: IntersectionFeature;
    movements: FeatureCollection;
    movementIdx: number;
  };

  function pickIntersection(e: CustomEvent<LayerClickInfo>) {
    let feature = e.detail.features[0] as IntersectionFeature;
    let movements = $backend!.getMovements(feature.properties.intersection_id);
    let movementIdx = 0;
    intersection = { feature, movements, movementIdx };
  }
</script>

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
        <li>
          <ModeLink mode={{ mode: "neighbourhood" }} />
        </li>
        <li>{pageTitle($mode.mode)}</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton mode={{ mode: "neighbourhood" }} />

    <h4>Roads</h4>
    <p>Click a road to visit its OSM object.</p>
    <div
      style="display: flex; align-items: start; justify-content: space-between;"
    >
      <h4>Intersections</h4>
      {#if intersection}
        <button class="close-btn" on:click={() => (intersection = null)}>
          Ⓧ
        </button>
      {/if}
    </div>
    {#if intersection}
      <PropertiesTable properties={intersection.feature.properties} />
      <h5>Turn restrictions:</h5>
      <PrevNext
        list={intersection.movements.features}
        bind:idx={intersection.movementIdx}
      />
    {:else}
      <p>Click an intersection to inspect its movements.</p>
    {/if}
  </div>

  <div slot="map">
    <RenderNeighbourhood>
      <HighlightBoundaryLayer />
      <CellLayer />
      <OneWayLayer />

      <CircleLayer
        {...layerId("debug-borders")}
        filter={["==", ["get", "kind"], "border_intersection"]}
        paint={{
          "circle-radius": 15,
          "circle-color": "green",
        }}
      >
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </CircleLayer>

      <NeighbourhoodRoadLayer
        interactive
        onClickLine={(f, _) => window.open(notNull(f.properties).way, "_blank")}
      >
        <div slot="line-popup">
          <Popup openOn="hover" let:props>
            <PropertiesTable properties={props} />
          </Popup>
        </div>
      </NeighbourhoodRoadLayer>
    </RenderNeighbourhood>

    <ModalFilterLayer interactive={true}>
      <!-- Note: This popup is currently broken (it was before this commit too). -->
      <Popup openOn="hover" let:props>
        <PropertiesTable properties={props} />
      </Popup>
    </ModalFilterLayer>

    <GeoJSON data={notNull($backend).getAllIntersections()} generateId>
      <CircleLayer
        {...layerId("debug-intersections")}
        paint={{
          "circle-radius": mapMetersToPixels(30),
          "circle-color": Style.mapFeature.hover.backgroundColor,
          "circle-stroke-color": Style.mapFeature.hover.backgroundColor,
          "circle-stroke-opacity": 0.8,
          "circle-stroke-width": hoverStateFilter(0, 3),
          "circle-opacity": [
            "case",
            [
              "==",
              ["get", "intersection_id"],
              intersection?.feature?.properties?.intersection_id ?? -1,
            ],
            0.3,
            hoverStateFilter(0.0, 0.5),
          ],
        }}
        manageHoverState
        hoverCursor="pointer"
        on:click={pickIntersection}
      >
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </CircleLayer>
    </GeoJSON>

    {#if intersection}
      <GeoJSON data={intersection.movements} generateId>
        <LineLayer
          {...layerId("debug-movements-outline")}
          paint={{
            "line-width": 2,
            "line-color": "red",
          }}
        />

        <FillLayer
          {...layerId("debug-movements-fill")}
          filter={["==", ["id"], intersection.movementIdx]}
          paint={{
            "fill-color": "cyan",
          }}
        />
      </GeoJSON>
    {/if}
  </div>
</SplitComponent>

<style>
  button.close-btn {
    padding: 4px;
    background: none;
    color: black;
    border: none;
    font-size: 120%;
  }
  button.close-btn:hover {
    background-color: rgba(0, 0, 0, 0.2);
  }
</style>
