<script lang="ts">
  import { Link } from "./common";
  import "@picocss/pico/css/pico.conditional.jade.min.css";
  import type { FeatureCollection, MultiPolygon, Polygon } from "geojson";
  import { onMount } from "svelte";
  import {
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    MapLibre,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { Popup } from "svelte-utils/map";
  import boundariesUrl from "../assets/cnt_boundaries.geojson?url";
  import { maptilerApiKey } from "./stores";

  let gj: FeatureCollection<
    Polygon | MultiPolygon,
    { kind: "LAD" | "REGION"; name: string }
  > = {
    type: "FeatureCollection" as const,
    features: [],
  };
  let ladNames: string[] = [];
  let regionNames: string[] = [];
  let kind = "LAD";

  onMount(async () => {
    let resp = await fetch(boundariesUrl);
    gj = await resp.json();

    for (let f of gj.features) {
      if (f.properties.kind == "LAD") {
        ladNames.push(f.properties.name);
      } else {
        regionNames.push(f.properties.name);
      }
    }
    ladNames.sort();
    regionNames.sort();
    ladNames = ladNames;
    regionNames = regionNames;
  });

  function onClick(e: CustomEvent<LayerClickInfo>) {
    let props = e.detail.features[0].properties!;
    newFile(`${props.kind}_${props.name}`);
  }

  function newFile(boundary: string) {
    let filename = "";
    while (true) {
      filename =
        window.prompt(
          `Please pick a project name to create in ${boundary}`,
          filename,
        ) || "";
      if (filename == "") {
        // If the user leaves this blank or presses cancel, stop prompting them.
        return;
      }
      let project = `ltn_cnt/${boundary}/${filename}`;
      if (window.localStorage.getItem(project) != null) {
        window.alert(
          `The project name ${filename} is already used; please pick another`,
        );
        continue;
      }

      // Create a blank project
      window.localStorage.setItem(
        project,
        JSON.stringify({
          type: "FeatureCollection",
          features: [],
          study_area_name: boundary,
        }),
      );

      window.location.href = `index.html?project=${encodeURIComponent(project)}`;
      return;
    }
  }

  // Returns boundary => list of filenames
  function listAllFiles(): Map<string, string[]> {
    let map = new Map();
    for (let i = 0; i < window.localStorage.length; i++) {
      let key = window.localStorage.key(i)!;
      if (key.startsWith("ltn_cnt/")) {
        try {
          let [_, boundary, filename] = key.split("/");
          if (!map.has(boundary)) {
            map.set(boundary, []);
          }
          map.get(boundary).push(filename);
        } catch (_) {}
      }
    }

    for (let list of map.values()) {
      list.sort();
    }
    return map;
  }
</script>

<div style="display: flex">
  <div class="pico left">
    <h2>Connected Neighbourhoods Tool</h2>

    <fieldset>
      <label>
        <input type="radio" value="LAD" bind:group={kind} />
        Local Authority Districts
      </label>
      <label>
        <input type="radio" value="REGION" bind:group={kind} />
        Regions
      </label>
    </fieldset>

    <p>Choose a boundary below or on the map to begin sketching:</p>
    <ul style="columns: 3">
      {#if kind == "LAD"}
        {#each ladNames as name}
          <li><Link on:click={() => newFile(`LAD_${name}`)}>{name}</Link></li>
        {/each}
      {:else}
        {#each regionNames as name}
          <li>
            <Link on:click={() => newFile(`REGION_${name}`)}>{name}</Link>
          </li>
        {/each}
      {/if}
    </ul>

    <hr />

    <p>Or continue with a previously opened file:</p>

    <div style="columns: 2">
      {#each listAllFiles() as [boundary, list]}
        <div class="group">
          <h2>{boundary}</h2>
          {#each list as filename}
            <p>
              <a href={`index.html?project=ltn_cnt/${boundary}/${filename}`}>
                {filename}
              </a>
            </p>
          {/each}
        </div>
      {/each}
    </div>

    <style>
      .group {
        border: 1px solid black;
        padding: 4px;
        margin-bottom: 8px;
        break-inside: avoid-column;
      }
    </style>
  </div>

  <div class="main">
    <div style="position: relative; width: 100%; height: 100%;">
      <MapLibre
        style={`https://api.maptiler.com/maps/streets-v2/style.json?key=${maptilerApiKey}`}
        standardControls
        bounds={[-8.943, 54.631, -0.901, 59.489]}
        on:error={(e) => {
          // @ts-expect-error ErrorEvent isn't exported
          console.log(e.detail.error);
        }}
      >
        <GeoJSON data={gj} generateId>
          <FillLayer
            filter={["==", ["get", "kind"], kind]}
            paint={{
              "fill-color": "rgb(200, 100, 240)",
              "fill-outline-color": "rgb(200, 100, 240)",
              "fill-opacity": hoverStateFilter(0.0, 0.5),
            }}
            beforeId="Road labels"
            manageHoverState
            hoverCursor="pointer"
            on:click={onClick}
          >
            <Popup openOn="hover" let:props>
              <p>{props.name}</p>
            </Popup>
          </FillLayer>

          <LineLayer
            filter={["==", ["get", "kind"], kind]}
            paint={{
              "line-color": "rgb(200, 100, 240)",
              "line-width": 2.5,
            }}
            beforeId="Road labels"
            manageHoverState
          />
        </GeoJSON>
      </MapLibre>
    </div>
  </div>
</div>

<style>
  .left {
    width: 35%;
    height: 100vh;
    overflow: scroll;
    padding: 8px;
  }

  .main {
    width: 65%;
    height: 100vh;
  }
</style>
