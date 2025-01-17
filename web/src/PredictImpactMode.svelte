<script lang="ts">
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { notNull } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { layerId, Link, roadLineWidth } from "./common";
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
        <li>Predict impact mode</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton on:click={() => ($mode = { mode: "network" })} />

    <p>This mode shows... TODO</p>
  </div>

  <div slot="map">
    <GeoJSON data={notNull($backend).predictImpact()} generateId>
      <LineLayer
        {...layerId("predict-impact")}
        paint={{
          "line-width": roadLineWidth(0),
          "line-color": [
            "case",
            ["<", ["get", "after"], ["get", "before"]],
            "green",
            "red",
          ],
        }}
      >
        <Popup openOn="hover" let:props>
          <p>
            {props.before.toLocaleString()} before, {props.after.toLocaleString()}
            after
          </p>
        </Popup>
      </LineLayer>
    </GeoJSON>
  </div>
</SplitComponent>
