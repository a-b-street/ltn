<script lang="ts">
  import { CircleLayer, GeoJSON, LineLayer } from "svelte-maplibre";
  import { notNull, PropertiesTable } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { layerId, Link } from "./common";
  import { CellLayer, HighlightBoundaryLayer, OneWayLayer } from "./layers";
  import RenderNeighbourhood from "./RenderNeighbourhood.svelte";
  import { backend, mode } from "./stores";
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
          <Link on:click={() => ($mode = { mode: "network" })}>
            Pick neighbourhood
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "neighbourhood" })}>
            Editing
          </Link>
        </li>
        <li>Debug mode</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton on:click={() => ($mode = { mode: "neighbourhood" })} />
  </div>

  <div slot="map">
    <HighlightBoundaryLayer gj={notNull($backend).renderNeighbourhood()} />

    <GeoJSON data={notNull($backend).renderNeighbourhood()} generateId>
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

      <LineLayer
        {...layerId("debug-crosses")}
        filter={["==", ["get", "kind"], "crosses"]}
        paint={{
          "line-width": 5,
          "line-color": "blue",
        }}
      >
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </LineLayer>
    </GeoJSON>

    <RenderNeighbourhood
      gj={notNull($backend).renderNeighbourhood()}
      interactive
      onClickLine={(f, _) => window.open(notNull(f.properties).way, "_blank")}
    >
      <div slot="line-popup">
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </div>
    </RenderNeighbourhood>

    <GeoJSON data={notNull($backend).renderModalFilters()} generateId>
      <CircleLayer
        {...layerId("debug-filters")}
        paint={{
          "circle-radius": 15,
          "circle-color": "black",
        }}
      >
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </CircleLayer>
    </GeoJSON>
  </div>
</SplitComponent>
