<script lang="ts">
  import { getContext } from "svelte";
  import { CircleLayer, hoverStateFilter, Popup } from "svelte-maplibre";
  import { layerId, mapMetersToPixels, Style } from "../common";
  import {
    Intersection,
    type IntersectionFeature,
  } from "../common/Intersection";
  import EditIntersectionPopup from "../edit/EditIntersectionPopup.svelte";
  import type { RenderNeighbourhoodOutput } from "../wasm";

  export let neighbourhood: RenderNeighbourhoodOutput =
    getContext("neighbourhoodGj");
  export let onClickIntersection = (intersection: Intersection) => {};
  export let interactive: boolean = false;
  export let show = true;

  /// NOTE: this takes the intersection's index in the neighborhood FeatureCollection, *not* the intersectionId!
  function getIntersectionByFeatureIndex(
    intersectionIdx: number,
  ): Intersection {
    // The Feature from maplibre has its nested properties serialized as a JSON string.
    // see: https://github.com/dimfeld/svelte-maplibre/discussions/162#discussioncomment-9295264
    //
    // So we refetch the corresponding IntersectionFeature from the neighbourhood to get all its properties.
    let intersectionFeature = neighbourhood.features[intersectionIdx];
    let intersection = new Intersection(
      intersectionFeature as IntersectionFeature,
    );
    return intersection;
  }

  function getIntersectionFromFeatures(features: any): Intersection {
    // Note: this `id` is autoassigned by maplibre, and corresponds to the index of the feature in the neighbourhood.
    return getIntersectionByFeatureIndex(features[0].id);
  }
</script>

<!-- 
 FIXME: If the user clicks twice to add and then immediately rotate a filter, they will be zoomed in.
 I haven't looked too deeply int how maplibre manages this, but none of the following prevents zooming...
 
     event.preventDefault();
     event.stopPropagation();
     event.stopImmediatePropagation();

 Maybe because `event.cancelable == false`
-->
<CircleLayer
  {...layerId("editable-intersections")}
  filter={["==", ["get", "kind"], "editable_intersection"]}
  paint={{
    "circle-radius": mapMetersToPixels(30),
    "circle-color": Style.mapFeature.hover.backgroundColor,
    "circle-opacity": hoverStateFilter(0.0, 0.5),
    "circle-stroke-color": Style.mapFeature.hover.backgroundColor,
    "circle-stroke-opacity": 0.8,
    "circle-stroke-width": hoverStateFilter(0, 3),
  }}
  layout={{
    visibility: show ? "visible" : "none",
  }}
  minzoom={13}
  {interactive}
  manageHoverState
  hoverCursor="pointer"
  on:click={(event) =>
    onClickIntersection(getIntersectionFromFeatures(event.detail.features))}
>
  <Popup openOn="hover" let:features>
    <EditIntersectionPopup
      intersection={getIntersectionFromFeatures(features)}
    />
  </Popup>
</CircleLayer>
