<script lang="ts">
  import { onMount } from "svelte";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { constructMatchExpression } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import {
    DotMarker,
    layerId,
    ModeLink,
    pageTitle,
    prettyPrintDistance,
    prettyPrintTime,
  } from "./common";
  import { ModalFilterType } from "./common/ModalFilterType";
  import {
    CellLayer,
    HighlightBoundaryLayer,
    ModalFilterLayer,
    NeighbourhoodRoadLayer,
    OneWayLayer,
  } from "./layers";
  import {
    backend,
    ensurePointInVisibleBounds,
    ignoreAutomatedBollards,
    mainRoadPenalty,
    mode,
    routePtA,
    routePtB,
  } from "./stores";

  interface Props {
    prevMode: "pick-neighbourhood" | "neighbourhood" | "impact-one-destination";
  }

  let { prevMode }: Props = $props();

  let neighbourhoodGj =
    prevMode == "neighbourhood" ? $backend!.renderNeighbourhood() : null;

  let gj = $derived(
    $backend!.compareRoute(
      $routePtA,
      $routePtB,
      $mainRoadPenalty,
      $ignoreAutomatedBollards,
    ),
  );
  let routeBefore = $derived(
    gj.features.find((f) => f.properties.kind == "before"),
  );
  let routeAfter = $derived(
    gj.features.find((f) => f.properties.kind == "after"),
  );

  onMount(() => {
    // There seems to be a race with the Marker component, so we wait just a bit before updating.
    setTimeout(() => {
      ensurePointInVisibleBounds(routePtA);
      ensurePointInVisibleBounds(routePtB);
    }, 10);
  });
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

    <p>Drag markers for a route</p>

    <u style:color="red">Route before changes</u>
    {#if routeBefore}
      <p>
        {prettyPrintDistance(routeBefore.properties.distance)}, {prettyPrintTime(
          routeBefore.properties.time,
        )}
      </p>
    {:else}
      <p>
        No possible route (
        <i>This is usually a known software bug</i>
        )
      </p>
    {/if}

    <u style:color="blue">Route after changes</u>
    {#if routeAfter}
      <p>
        {prettyPrintDistance(routeAfter.properties.distance)}, {prettyPrintTime(
          routeAfter.properties.time,
        )}
      </p>
    {:else}
      <p>
        No possible route (
        <i>This is usually a known software bug</i>
        )
      </p>
    {/if}

    <hr />

    <label>
      Slow-down factor for main roads: {$mainRoadPenalty}
      <input
        type="range"
        bind:value={$mainRoadPenalty}
        min="1.0"
        max="5.0"
        step="0.1"
      />

      <i>
        Increase to see how drivers may detour in heavy traffic. 1 means
        free-flow.
      </i>
    </label>

    <hr />

    <label>
      <input type="checkbox" bind:checked={$ignoreAutomatedBollards} />
      <img
        src={ModalFilterType.automatedBollard.iconURL}
        alt="Automated traffic bollard"
        width="20"
      />
      Ignore automated traffic bollards (for residents and other exemptions)
    </label>
  {/snippet}

  {#snippet main()}
    {#if prevMode == "neighbourhood"}
      <GeoJSON data={neighbourhoodGj!} generateId>
        <HighlightBoundaryLayer />
        <CellLayer />
        <OneWayLayer />
        <NeighbourhoodRoadLayer
          interactive={false}
          maxShortcuts={neighbourhoodGj!.maxShortcuts}
        />
      </GeoJSON>
    {/if}

    <ModalFilterLayer interactive={false} />

    <GeoJSON data={gj}>
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

    <DotMarker bind:lngLat={$routePtA} draggable>A</DotMarker>
    <DotMarker bind:lngLat={$routePtB} draggable>B</DotMarker>
  {/snippet}
</SplitComponent>
