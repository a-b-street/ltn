<script lang="ts">
  import {
    hoverStateFilter,
    CircleLayer,
    GeoJSON,
    LineLayer,
    Popup,
  } from "svelte-maplibre";
  import { notNull, PropertiesTable } from "./common";
  import RenderNeighbourhood from "./RenderNeighbourhood.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode } from "./stores";
</script>

<SplitComponent>
  <div slot="sidebar">
    <h1>Debug mode</h1>

    <div>
      <button on:click={() => ($mode = { mode: "neighbourhood" })}
        >Back to editing</button
      >
    </div>
  </div>

  <div slot="map">
    <RenderNeighbourhood
      gjInput={JSON.parse(notNull($app).renderNeighbourhood())}
      interactive
      onClickLine={(f) => window.open(notNull(f.properties).way, "_blank")}
    >
      <div slot="line-popup">
        <Popup openOn="hover" let:data>
          <PropertiesTable properties={notNull(data).properties} />
        </Popup>
      </div>
      <svelte:fragment slot="more-layers">
        <CircleLayer
          filter={["==", ["get", "kind"], "border_intersection"]}
          paint={{
            "circle-radius": 15,
            "circle-color": "green",
          }}
        >
          <Popup openOn="hover" let:data>
            <PropertiesTable properties={notNull(data).properties} />
          </Popup>
        </CircleLayer>
        <LineLayer
          filter={["==", ["get", "kind"], "crosses"]}
          paint={{
            "line-width": 5,
            "line-color": "blue",
          }}
        >
          <Popup openOn="hover" let:data>
            <PropertiesTable properties={notNull(data).properties} />
          </Popup>
        </LineLayer>
      </svelte:fragment>
    </RenderNeighbourhood>

    <GeoJSON data={JSON.parse(notNull($app).renderModalFilters())} generateId>
      <CircleLayer
        paint={{
          "circle-radius": 15,
          "circle-color": "black",
        }}
      >
        <Popup openOn="hover" let:data>
          <PropertiesTable properties={notNull(data).properties} />
        </Popup>
      </CircleLayer>
    </GeoJSON>

    <GeoJSON data={JSON.parse(notNull($app).snapperSplits())} generateId>
      <LineLayer
        filter={["==", ["get", "kind"], "split road"]}
        paint={{
          "line-width": 5,
          "line-color": "red",
          "line-opacity": hoverStateFilter(0.5, 1.0),
        }}
        manageHoverState
      >
        <Popup openOn="hover" let:data>
          <PropertiesTable properties={notNull(data).properties} />
        </Popup>
      </LineLayer>
      <CircleLayer
        filter={["==", ["get", "kind"], "split point"]}
        paint={{
          "circle-radius": 15,
          "circle-color": "black",
        }}
      >
        <Popup openOn="hover" let:data>
          <PropertiesTable properties={notNull(data).properties} />
        </Popup>
      </CircleLayer>
    </GeoJSON>
  </div>
</SplitComponent>
