<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import type { AreaProps } from "route-snapper-ts";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { ModeLink, pageTitle } from "./common";
  import AreaControls from "./common/draw_area/AreaControls.svelte";
  import { type Waypoint } from "./common/draw_area/stores";
  import { backend, map, mode, saveCurrentProject } from "./stores";

  interface Props {
    name: string;
    existing: Feature<Polygon, AreaProps>;
  }

  let { name, existing }: Props = $props();
  let waypoints: Waypoint[] = $state([]);
  let drawnShape: Feature<Polygon> | undefined = $state();

  let unformattedWaypoints = existing.properties.waypoints;
  if (!unformattedWaypoints) {
    // No stored waypoints -- this is either a boundary drawn with a very old
    // version of this tool, or an auto-generated boundary.
    // "backfill" by using the geometry (simplified) as freehand points.

    // Note the second polygon ring is used, because the boundary is expressed as
    // "everywhere" minus a hole for the boundary, to achieve the fade-outside effect.
    let neighbourhoodBoundary = {
      type: "LineString" as const,
      coordinates: existing.geometry.coordinates[1],
    };

    unformattedWaypoints = $backend!.extractWaypointsFromRing(
      neighbourhoodBoundary,
    );
  }
  // Transform into the correct format
  waypoints = unformattedWaypoints.map((waypt) => {
    return {
      point: [waypt.lon, waypt.lat],
      snapped: waypt.snapped,
    };
  });

  function finish() {
    if (drawnShape) {
      try {
        $backend!.setCurrentNeighbourhoodBoundary(name, drawnShape);
        saveCurrentProject();
        $mode = { mode: "neighbourhood" };
      } catch (err) {
        window.alert(`Sorry, this boundary is invalid: ${err}`);
        cancel();
      }
    }
  }

  function cancel() {
    $mode = {
      mode: "neighbourhood",
    };
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

    <h1>Adjust boundary</h1>

    <AreaControls map={$map!} bind:waypoints bind:drawnShapeOut={drawnShape} />

    <div style="display: flex; gap: 16px;">
      <button onclick={finish} disabled={waypoints.length < 3}>Finish</button>
      <button class="secondary" onclick={cancel}>Cancel</button>
    </div>
  {/snippet}
</SplitComponent>
