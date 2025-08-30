<script lang="ts">
  import type { FeatureCollection } from "geojson";
  import {
    CircleLayer,
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    Popup,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { PropertiesTable } from "svelte-utils";
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
  } from "./layers";
  import ModalFilterLayer from "./layers/ModalFilterLayer.svelte";
  import { backend, mode } from "./stores";

  let neighbourhoodGj = $backend!.renderNeighbourhood();

  let intersection: DebugIntersection | null = $state(null);
  type DebugIntersection = {
    feature: IntersectionFeature;
    movements: FeatureCollection;
    movementIdx: number;
  };

  function pickIntersection(e: LayerClickInfo) {
    let feature = e.features[0] as IntersectionFeature;
    let movements = $backend!.getMovements(feature.properties.intersection_id);
    let movementIdx = 0;
    intersection = { feature, movements, movementIdx };
  }
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
        <li>
          <ModeLink mode={{ mode: "neighbourhood" }} />
        </li>
        <li>{pageTitle($mode.mode)}</li>
      </ul>
    </nav>
  {/snippet}

  {#snippet left()}
    <BackButton mode={{ mode: "neighbourhood" }} />

    <h4>Roads</h4>
    <p>Click a road to visit its OSM object.</p>
    <div
      style="display: flex; align-items: start; justify-content: space-between;"
    >
      <h4>Intersections</h4>
      {#if intersection}
        <button class="close-btn" onclick={() => (intersection = null)}>
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
  {/snippet}

  {#snippet main()}
    <GeoJSON data={neighbourhoodGj} generateId>
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
        <Popup openOn="hover">
          {#snippet children({ data })}
            <PropertiesTable properties={data!.properties!} />
          {/snippet}
        </Popup>
      </CircleLayer>

      <NeighbourhoodRoadLayer
        interactive
        onClickLine={(f, _) => window.open(f.properties!.way, "_blank")}
        maxShortcuts={neighbourhoodGj.maxShortcuts}
      >
        {#snippet linePopup()}
          <Popup openOn="hover">
            {#snippet children({ data })}
              <PropertiesTable properties={data!.properties!} />
            {/snippet}
          </Popup>
        {/snippet}
      </NeighbourhoodRoadLayer>
    </GeoJSON>

    <ModalFilterLayer interactive={true}>
      {#snippet modalFilterPopup()}
        <Popup openOn="hover">
          {#snippet children({ data })}
            <PropertiesTable properties={data!.properties!} />
          {/snippet}
        </Popup>
      {/snippet}

      {#snippet turnRestrictionPopup()}
        <Popup openOn="hover">
          {#snippet children({ data })}
            <PropertiesTable properties={data!.properties!} />
          {/snippet}
        </Popup>
      {/snippet}
    </ModalFilterLayer>

    <GeoJSON data={$backend!.getAllIntersections()} generateId>
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
        onclick={pickIntersection}
      >
        <Popup openOn="hover">
          {#snippet children({ data })}
            <PropertiesTable properties={data!.properties!} />
          {/snippet}
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
  {/snippet}
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
