<script lang="ts">
  import type { Feature, FeatureCollection, LineString } from "geojson";
  import {
    Eraser,
    Paintbrush,
    Pointer,
    Redo,
    Trash2,
    Undo,
  } from "lucide-svelte";
  import type { LngLat, MapMouseEvent } from "maplibre-gl";
  import { onDestroy, onMount } from "svelte";
  import {
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    MapEvents,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { notNull, SequentialLegend } from "svelte-utils";
  import { emptyGeojson, Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import eraserCursorURL from "../../assets/cursors/eraser.svg?url";
  import paintbrushCursorURL from "../../assets/cursors/paintbrush.svg?url";
  import noLeftUrl from "../../assets/filters/no_left_turn.png?url";
  import noRightUrl from "../../assets/filters/no_right_turn.png?url";
  import noStraightUrl from "../../assets/filters/no_straight_turn.png?url";
  import noUTurnRtlUrl from "../../assets/filters/no_u_left_to_right_turn.png?url";
  import noUTurnLtrUrl from "../../assets/filters/no_u_right_to_left_turn.png?url";
  import onewayArrowUrl from "../../assets/one_way_left.svg?url";
  import mainRoadIconUrl from "../../assets/traffic_1.svg?url";
  import AnimatePaths from "../AnimatePaths.svelte";
  import {
    HelpButton,
    initTooltips,
    layerId,
    ModeLink,
    pageTitle,
    roadLineWidth,
  } from "../common";
  import { speedColorScale, speedLimits } from "../common/colors";
  import type { Intersection } from "../common/Intersection";
  import { ModalFilterType } from "../common/ModalFilterType";
  import NeighbourhoodBoundarySummary from "../common/NeighbourhoodBoundarySummary.svelte";
  import {
    CellLayer,
    HighlightBoundaryLayer,
    ModalFilterLayer,
    NeighbourhoodRoadLayer,
    OneWayLayer,
    RenderNeighbourhood,
  } from "../layers";
  import EditableIntersectionLayer from "../layers/EditableIntersectionLayer.svelte";
  import {
    animateShortcuts,
    backend,
    currentFilterType,
    devMode,
    drawBorderEntries,
    map,
    mode,
    mutationCounter,
    roadStyle,
    saveCurrentProject,
    thickRoadsForShortcuts,
  } from "../stores";
  import type {
    AllShortcuts,
    NeighbourhoodBoundaryFeature,
    RenderNeighbourhoodOutput,
  } from "../wasm";
  import ChangeFilterModal from "./ChangeFilterModal.svelte";
  import FreehandLine from "./FreehandLine.svelte";

  // Caller is responsible for doing backend.setCurrentNeighbourhood

  type Action =
    | { kind: "filter"; freehand: boolean }
    | { kind: "oneway" }
    | {
        kind: "turn_restriction";
        from_road_id: number | null;
        from_road_name: string;
        possible_targets: FeatureCollection<
          LineString,
          { road: number; name: string }
        >;
      }
    | {
        kind: "main-roads";
        tool: "toggle" | "freehand-main" | "freehand-erase";
      };
  function startTurnRestrictionAction(): Action {
    return {
      kind: "turn_restriction",
      from_road_id: null,
      from_road_name: "",
      possible_targets: emptyGeojson() as FeatureCollection<
        LineString,
        { road: number; name: string }
      >,
    };
  }
  let action: Action = { kind: "filter", freehand: false };

  $: if (action.kind == "oneway" || action.kind == "main-roads") {
    $map!.doubleClickZoom.disable();
  } else {
    $map!.doubleClickZoom.enable();
  }

  $: {
    if (
      (action.kind == "filter" && action.freehand) ||
      (action.kind == "main-roads" && action.tool == "freehand-main")
    ) {
      $map!.getCanvas().style.cursor = `url(${paintbrushCursorURL}) 8 22, cell`;
    } else if (action.kind == "main-roads" && action.tool == "freehand-erase") {
      $map!.getCanvas().style.cursor = `url(${eraserCursorURL}) 8 22, cell`;
    } else {
      $map!.getCanvas().style.cursor = "";
    }
  }

  let settingFilterType = false;
  let undoLength = 0;
  let redoLength = 0;
  let boundary: NeighbourhoodBoundaryFeature | null;

  let gj: RenderNeighbourhoodOutput;
  $: rerender($mutationCounter);

  let allShortcuts = emptyGeojson() as AllShortcuts;
  let lastShortcutCalculation = 0;
  $: recalculateShortcuts($mutationCounter, $animateShortcuts);

  $: numDisconnectedCells = gj.features.filter(
    (f) =>
      f.properties.kind == "cell" && f.properties.cell_color == "disconnected",
  ).length;

  onMount(() => {
    initTooltips();
  });
  onDestroy(() => {
    $map!.doubleClickZoom.enable();
  });

  function rerender(_x: number) {
    gj = $backend!.renderNeighbourhood();
    // @ts-expect-error TS can't figure out that we're narrowing the case here
    boundary = gj.features.find((f) => f.properties.kind == "boundary")!;

    undoLength = gj.undo_length;
    redoLength = gj.redo_length;

    saveCurrentProject();
  }

  function recalculateShortcuts(_x: number, animate: boolean) {
    if ($mutationCounter == lastShortcutCalculation || !animate) {
      return;
    }
    allShortcuts = $backend!.getAllShortcuts();
    lastShortcutCalculation = $mutationCounter;
  }

  function onClickLine(f: Feature, pt: LngLat) {
    if (action.kind == "filter") {
      $backend!.addModalFilter(pt, $currentFilterType);
      $mutationCounter++;
    } else if (action.kind == "oneway") {
      $backend!.toggleTravelFlow(f.properties!.road);
      $mutationCounter++;
    } else if (action.kind == "main-roads" && action.tool == "toggle") {
      $backend!.toggleMainRoad(f.properties!.road);
      $mutationCounter++;
    } else if (action.kind == "turn_restriction") {
      action.from_road_id = f.properties!.road;
      action.from_road_name = f.properties!.name || "unnamed road";
      action.possible_targets = $backend!.getTurnRestrictionTargets(
        f.properties!.road,
      );
      // TODO The blue hover state gets stuck
    }
  }

  function onMapClick(e: CustomEvent<MapMouseEvent>) {
    if (action.kind != "turn_restriction") {
      return;
    }

    // If we click a blank area, reset some state. Not sure why, but clicking
    // layers doesn't always prevent a click on the map itself.
    if (
      $map!.queryRenderedFeatures(e.detail.point, {
        layers: ["interior-roads", "turn-restriction-targets"],
      }).length > 0
    ) {
      return;
    }

    action = startTurnRestrictionAction();
  }

  function createTurnRestriction(e: CustomEvent<LayerClickInfo>) {
    if (action.kind == "turn_restriction" && action.from_road_id != null) {
      let to = e.detail.features[0].properties!.road;
      $backend!.addTurnRestriction(action.from_road_id, to);
      $mutationCounter++;
    }
    action = startTurnRestrictionAction();
  }

  function deleteModalFilter(e: CustomEvent<LayerClickInfo>) {
    let f = e.detail.features[0];
    $backend!.deleteModalFilter(f.properties!.road);
    $mutationCounter++;
  }

  function deleteTurnRestriction(e: CustomEvent<LayerClickInfo>) {
    let f = e.detail.features[0];
    $backend!.deleteTurnRestriction(
      f.properties!.intersection,
      f.properties!.from_road,
      f.properties!.to_road,
    );
    $mutationCounter++;
  }

  function onClickIntersection(intersection: Intersection) {
    if (action.kind != "filter") {
      console.assert(
        false,
        `this shouldn't happen - intersections should only be clickable when in 'filter' mode, not ${action.kind}`,
      );
      return;
    }

    if (intersection.hasRotatedFilter) {
      $backend!.deleteDiagonalFilter(intersection);
      $mutationCounter++;
    } else if (intersection.filter) {
      $backend!.rotateDiagonalFilter(intersection);
      $mutationCounter++;
    } else {
      $backend!.addDiagonalFilter(intersection);
      $mutationCounter++;
    }
  }

  function onKeyDown(e: KeyboardEvent) {
    // Ignore keypresses if we're not focused on the map
    if ((e.target as HTMLElement).tagName == "INPUT") {
      return;
    }
    if (e.ctrlKey && e.key == "z") {
      undo();
    }
    if (e.ctrlKey && e.key == "y") {
      redo();
    }
    if (e.key == "1") {
      action = { kind: "filter", freehand: false };
    }
    if (e.key == "2") {
      action = { kind: "oneway" };
    }
    if (e.key == "3") {
      action = startTurnRestrictionAction();
    }
    if (e.key == "4") {
      action = { kind: "main-roads", tool: "toggle" };
    }
  }

  function undo() {
    $backend!.undo();
    $mutationCounter++;
  }
  function redo() {
    $backend!.redo();
    $mutationCounter++;
  }

  function paintedModalFiltersLine(e: CustomEvent<Feature<LineString> | null>) {
    let f = e.detail;
    if (f) {
      $backend!.addManyModalFilters(f, $currentFilterType);
      $mutationCounter++;
    }
    action = { kind: "filter", freehand: false };
  }

  function paintedRoadClassificationsLine(
    e: CustomEvent<Feature<LineString> | null>,
    addToUndoStack: boolean,
  ) {
    if (action.kind != "main-roads") {
      console.assert(false, "cant paint line unless in main-roads mode");
      return;
    }
    let f = e.detail;
    if (f) {
      $backend!.reclassifyRoadsAlongLine(
        f,
        action.tool == "freehand-main",
        addToUndoStack,
      );
      $mutationCounter++;
    }
  }

  function eraseAllMainRoads() {
    $backend!.eraseAllMainRoads();
    $mutationCounter++;
  }

  let shortcutDescriptionText =
    "Shortcuts are routes from one main road to another, which cut through the neighborhood's interior.";
  let cellsDescriptionText =
    "Cells are the colored area reachable without travelling along a main road.";

  let turnRestrictionUrls: Record<any, string> = {
    left: noLeftUrl,
    right: noRightUrl,
    straight: noStraightUrl,
    u_left_to_right: noUTurnLtrUrl,
    u_right_to_left: noUTurnRtlUrl,
  };
</script>

<svelte:window on:keydown={onKeyDown} />

<SplitComponent>
  <div slot="top" style="display: flex; justify-content: space-between;">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <ModeLink mode={{ mode: "title" }} />
        </li>
        <li>
          <ModeLink mode={{ mode: "pick-neighbourhood" }} />
        </li>
        <li>
          {pageTitle($mode.mode)}
          <HelpButton>
            <p>
              Now that you've defined a neighbourhood boundary, you can see the
              possible shortcuts that vehicles are currently able to take
              through it. You can add a new modal filter to try and solve this.
              The colored "cell" areas show what's reachable for drivers without
              leaving the boundary you've drawn.
            </p>
          </HelpButton>
        </li>
      </ul>
    </nav>
    <nav>
      <ul>
        <li>
          <ModeLink mode={{ mode: "view-shortcuts" }} />
        </li>
        <li>
          <ModeLink mode={{ mode: "route", prevMode: "neighbourhood" }} />
        </li>
        <li>
          <ModeLink
            mode={{ mode: "predict-impact", prevMode: "neighbourhood" }}
          />
        </li>
        <li>
          <ModeLink mode={{ mode: "impact-one-destination" }} />
        </li>
        <li>
          <ModeLink
            mode={{
              mode: "set-boundary",
              name: notNull(boundary).properties.name,
              existing: notNull(boundary),
            }}
          />
        </li>
        {#if $devMode}
          <li>
            <ModeLink mode={{ mode: "debug-neighbourhood" }}>Debug</ModeLink>
          </li>
        {/if}
      </ul>
    </nav>
  </div>
  <div slot="sidebar">
    <h2>Editing tools</h2>
    <div
      class="tool-palette"
      style="display: flex; justify-content: space-between; flex-wrap: wrap; gap: 6px;"
    >
      <div
        style="height: 50px; display: flex; justify-content: left; gap: 6px;"
      >
        <button
          on:click={() => (action = { kind: "filter", freehand: false })}
          class="icon-btn"
          class:active={action.kind == "filter"}
          data-tippy-content="Add a modal filter (hotkey 1)"
        >
          <img
            src={notNull(ModalFilterType.getFilter($currentFilterType)).iconURL}
            alt="Add a modal filter"
          />
        </button>
        <button
          on:click={() => (action = { kind: "oneway" })}
          class="icon-btn"
          class:active={action.kind == "oneway"}
          data-tippy-content="Toggle one-way (hotkey 2)"
        >
          <!-- 
         cheat the default padding just a bit with negative placement, 
         these small circles crowd each other more than they crowd their container
         -->
          <div style="height: 100%; width: 100%; position: relative;">
            <img
              style="position: absolute; width: 60%; height: 60%; top: -1px; left: -1px;"
              src={onewayArrowUrl}
              alt="Reverse directions"
            />
            <img
              style="position: absolute; width: 60%; height: 60%; bottom: -1px; right: -1px; transform: rotate(180deg);"
              src={onewayArrowUrl}
              alt="Reverse directions"
            />
          </div>
        </button>
        <button
          on:click={() => (action = startTurnRestrictionAction())}
          class="icon-btn"
          class:active={action.kind == "turn_restriction"}
          data-tippy-content="Restrict turns (hotkey 3)"
        >
          <img src={noRightUrl} alt="Restrict turns" />
        </button>
        <button
          on:click={() => (action = { kind: "main-roads", tool: "toggle" })}
          class="icon-btn"
          class:active={action.kind == "main-roads"}
          data-tippy-content="Reclassify main roads (hotkey 4)"
        >
          <img src={mainRoadIconUrl} alt="Change main/minor roads" />
        </button>
      </div>
      <div
        style="height: 50px; display: flex; justify-content: right; gap: 6px;"
      >
        <button
          class="outline icon-btn"
          disabled={undoLength == 0}
          on:click={undo}
          data-tippy-content={undoLength == 0
            ? "Undo Ctrl+Z"
            : `Undo (${undoLength}) Ctrl+Z`}
        >
          <Undo />
        </button>
        <button
          class="outline icon-btn"
          disabled={redoLength == 0}
          on:click={redo}
          data-tippy-content={redoLength == 0
            ? "Redo Ctrl+Y"
            : `Redo (${redoLength}) Ctrl+Y`}
        >
          <Redo />
        </button>
      </div>
    </div>

    <div
      style="min-height: 200px; padding-bottom: 16px; border-bottom: solid var(--pico-muted-border-color) 1px;"
    >
      {#if action.kind == "filter"}
        <h3>Add modal filter</h3>
        <p>
          Modal filters restrict what kind of traffic can pass through a road
          segment. Place them strategically to deter shortcuts through your
          neighbourhood.
        </p>
        <ChangeFilterModal bind:show={settingFilterType} />
        <div
          style="display: flex; gap: 8px; align-items: leading; flex-direction: column; width: fit-content;"
        >
          <button class="outline" on:click={() => (settingFilterType = true)}>
            Change modal filter type
          </button>

          <button
            on:click={action.freehand
              ? () => (action = { kind: "filter", freehand: false })
              : () => (action = { kind: "filter", freehand: true })}
            class:active={action.freehand}
            class:outline={!action.freehand}
            data-tippy-content="Add many modal filters along a line"
          >
            <div style="display: flex; align-items: center; gap: 8px;">
              <Paintbrush />
              <span>Add along a line</span>
            </div>
          </button>
        </div>
      {:else if action.kind == "oneway"}
        <h3>Toggle one-way</h3>
        <p>
          Click on a road segment to toggle its direction. This will change the
          direction of traffic flow on that road.
        </p>
      {:else if action.kind == "turn_restriction"}
        <h3>Restrict turns</h3>
        <p>
          To restrict certain turns, first click on the source road, then the
          destination road. Traffic will no longer be able to turn from the
          source road to the destination road.
        </p>
      {:else if action.kind == "main-roads"}
        <h3>Reclassify main roads</h3>
        <p>
          <i>Main roads</i>, drawn in grey, were classified automatically using
          data from
          <a href="https://openstreetmap.org/about" target="_blank"
            >OpenStreetMap</a
          >, but you can reclassify a road segment by clicking on it.
        </p>

        <p>
          Main roads are typically better suited to support higher levels of
          traffic than neighbourhood roads.
        </p>
        <div
          class="classification-buttons"
          style="display: flex; flex-direction: column; gap: 8px; justify-content: left;"
        >
          <button
            on:click={() => {
              action = { kind: "main-roads", tool: "toggle" };
            }}
            class:active={action.tool == "toggle"}
            class:outline={action.tool != "toggle"}
            data-tippy-content="Click a road to reclassify it"
          >
            <div style="display: flex; align-items: center; gap: 8px;">
              <Pointer />
              <span>Toggle segment or pan map</span>
            </div>
          </button>
          <button
            on:click={() => {
              action = { kind: "main-roads", tool: "freehand-main" };
            }}
            class:active={action.tool == "freehand-main"}
            class:outline={action.tool != "freehand-main"}
            data-tippy-content="Reclassify multiple roads by drawing a line along them"
          >
            <div style="display: flex; align-items: center; gap: 8px;">
              <Paintbrush />
              <span>Mark as main along a line</span>
            </div>
          </button>
          <button
            on:click={() => {
              action = { kind: "main-roads", tool: "freehand-erase" };
            }}
            class:active={action.tool == "freehand-erase"}
            class:outline={action.tool != "freehand-erase"}
            data-tippy-content="Reclassify multiple roads by drawing a line along them"
          >
            <div style="display: flex; align-items: center; gap: 8px;">
              <Eraser />
              <span>Erase main classification</span>
            </div>
          </button>
          <button class:outline={true} on:click={eraseAllMainRoads}>
            <div style="display: flex; align-items: center; gap: 8px;">
              <Trash2 />
              <span>Erase all main roads</span>
            </div>
          </button>
        </div>
      {/if}
    </div>

    {#if numDisconnectedCells > 0}
      <mark>
        Some parts of the neighbourhood aren't reachable by drivers, shown in
        red
      </mark>
    {/if}

    <h2>Map style</h2>
    <label>
      <input type="checkbox" bind:checked={$animateShortcuts} />
      Animate shortcuts<span
        class="footnote-ref"
        data-tippy-content={shortcutDescriptionText}>1</span
      >
    </label>

    <label>
      <input type="checkbox" bind:checked={$drawBorderEntries} />
      Show entries into cells<span
        class="footnote-ref"
        data-tippy-content={cellsDescriptionText}>2</span
      >
    </label>

    <label>
      <input type="checkbox" bind:checked={$thickRoadsForShortcuts} />
      Road thickness depends on shortcuts<span
        class="footnote-ref"
        data-tippy-content={shortcutDescriptionText}>1</span
      >
    </label>

    <label
      style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;"
    >
      <span style="text-wrap: nowrap;">Draw roads:</span>
      <select
        style="margin: 0; padding: 8px; width: auto;"
        bind:value={$roadStyle}
      >
        <option value="shortcuts">Worst shortcuts</option>
        <option value="cells">Cell</option>
        <option value="edits">Edits (either filter or direction)</option>
        <option value="speeds">Speed limit</option>
      </select>
    </label>
    {#if $roadStyle == "speeds"}
      <SequentialLegend
        colorScale={speedColorScale}
        labels={{ limits: speedLimits }}
      />
    {/if}
    <h2>Neighbourhood stats</h2>
    <NeighbourhoodBoundarySummary neighbourhoodBoundary={notNull(boundary)} />
  </div>

  <div slot="map">
    <MapEvents on:click={onMapClick} />

    <RenderNeighbourhood input={gj}>
      <HighlightBoundaryLayer />
      <CellLayer />
      <OneWayLayer />

      <NeighbourhoodRoadLayer
        interactive={(action.kind == "filter" && !action.freehand) ||
          action.kind == "oneway" ||
          (action.kind == "main-roads" && action.tool == "toggle") ||
          (action.kind == "turn_restriction" && action.from_road_id == null)}
        {onClickLine}
      >
        <div slot="line-popup">
          <Popup openOn="hover" let:props>
            {#if props.kind == "interior_road"}
              <p>
                {props.shortcuts} shortcuts through {props.name ??
                  "unnamed road"}
                ({Math.round(props.speed_mph)} mph)
              </p>
            {:else if props.kind == "main_road"}
              <p>
                Main road: {props.name ?? "unnamed road"}
                ({Math.round(props.speed_mph)} mph)
              </p>
            {/if}
            {#if action.kind == "filter"}
              <div>
                <img
                  src={notNull(ModalFilterType.getFilter($currentFilterType))
                    .iconURL}
                  width="20"
                  alt="Add modal filter"
                />
                Click to add modal filter
              </div>
            {:else if action.kind == "oneway"}
              <p>Click to change direction</p>
            {:else if action.kind == "main-roads" && action.tool == "toggle"}
              <p>Click to designate a main road or not</p>
            {:else if action.kind == "turn_restriction"}
              <p>Click to create a turn restriction from here</p>
            {/if}
          </Popup>
        </div>
      </NeighbourhoodRoadLayer>
      <EditableIntersectionLayer
        interactive={action.kind == "filter"}
        neighbourhood={gj}
        {onClickIntersection}
      />
    </RenderNeighbourhood>

    {#if $animateShortcuts}
      <AnimatePaths paths={allShortcuts} />
    {/if}

    <ModalFilterLayer
      onClickModalFilter={deleteModalFilter}
      onClickTurnRestriction={deleteTurnRestriction}
      interactive={action.kind == "filter"}
    >
      <div slot="modal-filter">
        <Popup openOn="hover">Click to delete filter</Popup>
      </div>
      <div slot="turn-restriction">
        <Popup openOn="hover">Click to delete turn restriction</Popup>
      </div>
    </ModalFilterLayer>

    {#if action.kind == "filter" && action.freehand}
      <FreehandLine map={notNull($map)} on:done={paintedModalFiltersLine} />
    {:else if action.kind == "main-roads" && (action.tool == "freehand-main" || action.tool == "freehand-erase")}
      <FreehandLine
        map={notNull($map)}
        on:done={(e) => paintedRoadClassificationsLine(e, true)}
        on:progress={(e) => paintedRoadClassificationsLine(e, false)}
      />
    {/if}

    {#if action.kind == "turn_restriction"}
      <GeoJSON data={action.possible_targets || emptyGeojson()} generateId>
        <LineLayer
          {...layerId("turn-restriction-targets")}
          manageHoverState
          paint={{
            "line-color": "yellow",
            "line-opacity": hoverStateFilter(0.5, 1.0),
            "line-width": roadLineWidth(1),
          }}
          on:click={createTurnRestriction}
        >
          <Popup openOn="hover" let:props>
            <div>
              <img
                src={turnRestrictionUrls[props.kind]}
                width="20"
                alt="Add turn restriction"
              />

              Create a turn restriction from {action.from_road_name} to {props.name ||
                "unnamed road"}
            </div>
          </Popup>
        </LineLayer>
      </GeoJSON>
    {/if}
  </div>
</SplitComponent>

<style>
  .classification-buttons {
    width: fit-content;
  }

  .footnote-ref {
    font-size: 70%;
    color: var(--pico-secondary);
    text-decoration: underline;
    cursor: help;
    position: relative;
    top: -12px;
    left: 4px;
  }
</style>
