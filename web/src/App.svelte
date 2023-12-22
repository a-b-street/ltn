<script lang="ts">
  import turfBbox from "@turf/bbox";
  import { LTN } from "backend";
  import type { Feature, Polygon } from "geojson";
  import type { Map } from "maplibre-gl";
  import { MapLibre } from "svelte-maplibre";
  import { Layout } from "./common";
  import { RouteTool } from "./common/route_tool";
  import RouteSnapperLayer from "./common/RouteSnapperLayer.svelte";
  import MapLoader from "./MapLoader.svelte";
  import NeighbourhoodLayer from "./NeighbourhoodLayer.svelte";
  import NeighbourhoodMode from "./NeighbourhoodMode.svelte";
  import NetworkLayer from "./NetworkLayer.svelte";
  import ViewShortcutsLayer from "./ViewShortcutsLayer.svelte";
  import ViewShortcutsMode from "./ViewShortcutsMode.svelte";

  let offlineMode = true;
  let mapStyle = offlineMode
    ? {
        version: 8,
        sources: {},
        layers: [],
      }
    : "https://api.maptiler.com/maps/dataviz/style.json?key=MZEJTanw3WpxRvt7qDfo";

  type Mode =
    | {
        mode: "network";
      }
    | {
        mode: "set-boundary";
      }
    | {
        mode: "neighbourhood";
        boundary: Feature<Polygon>;
        addingFilter: boolean;
        undoLength: number;
        redoLength: number;
        rerender: number;
      }
    | {
        mode: "view-shortcuts";
        prevMode: Mode;
      };

  let mode = {
    mode: "network",
  };
  let app: LTN | undefined = undefined;
  let route_tool: RouteTool | undefined = undefined;
  let map: Map;

  function zoomToFit() {
    if (map && app) {
      // TODO wasteful
      let bbox = turfBbox(JSON.parse(app.render()));
      map.fitBounds(bbox, { animate: false });
    }
  }

  function gotApp(_x: LTN) {
    if (!app) {
      return;
    }
    console.log("New map model loaded");
    zoomToFit();
    mode = {
      mode: "network",
    };
    route_tool = new RouteTool(map, app.toRouteSnapper());
  }
  $: gotApp(app);

  function setBoundaryMode() {
    if (mode.mode == "network") {
      route_tool.startArea();
    } else if (mode.mode == "neighbourhood") {
      route_tool.editExistingArea(mode.boundary);
    }

    mode = {
      mode: "set-boundary",
    };
    route_tool.addEventListenerSuccess((feature) => {
      mode = {
        mode: "neighbourhood",
        boundary: feature,
        addingFilter: false,
        undoLength: 0,
        redoLength: 0,
        rerender: 0,
      };
      route_tool.clearEventListeners();
    });
    route_tool.addEventListenerFailure(() => {
      mode = {
        mode: "network",
      };
      route_tool.clearEventListeners();
    });
  }

  function devMode() {
    mode = {
      mode: "neighbourhood",
      boundary: {
        geometry: {
          coordinates: [
            [
              [-2.582766, 51.455751],
              [-2.582715, 51.455655],
              [-2.582477, 51.455207],
              [-2.582446, 51.455157],
              [-2.582409, 51.455129],
              [-2.582334, 51.455111],
              [-2.58226, 51.455115],
              [-2.582174, 51.455113],
              [-2.582115, 51.455126],
              [-2.581976, 51.454882],
              [-2.581812, 51.454602],
              [-2.582181, 51.454505],
              [-2.582091, 51.454224],
              [-2.582071, 51.454159],
              [-2.582042, 51.454073],
              [-2.581915, 51.453674],
              [-2.581972, 51.453287],
              [-2.581714, 51.45322],
              [-2.58169, 51.453214],
              [-2.581866, 51.452966],
              [-2.581588, 51.45288],
              [-2.581521, 51.45281],
              [-2.581584, 51.452731],
              [-2.581401, 51.452671],
              [-2.581103, 51.452566],
              [-2.579862, 51.452092],
              [-2.579808, 51.452069],
              [-2.579753, 51.452045],
              [-2.579074, 51.451752],
              [-2.578228, 51.45135],
              [-2.578131, 51.451293],
              [-2.578066, 51.451251],
              [-2.577641, 51.450924],
              [-2.577504, 51.450818],
              [-2.577228, 51.450628],
              [-2.576815, 51.450348],
              [-2.576749, 51.450294],
              [-2.576487, 51.450084],
              [-2.57599, 51.449653],
              [-2.575886, 51.449685],
              [-2.575807, 51.449709],
              [-2.575324, 51.44978],
              [-2.574139, 51.449954],
              [-2.573887, 51.450007],
              [-2.573699, 51.450097],
              [-2.573534, 51.450262],
              [-2.573415, 51.450398],
              [-2.573305, 51.45055],
              [-2.573286, 51.450604],
              [-2.573279, 51.450637],
              [-2.573275, 51.45066],
              [-2.57327, 51.450705],
              [-2.573278, 51.450761],
              [-2.573366, 51.451076],
              [-2.573491, 51.451385],
              [-2.573553, 51.451514],
              [-2.573591, 51.451605],
              [-2.573635, 51.451726],
              [-2.573785, 51.451943],
              [-2.573862, 51.452052],
              [-2.574029, 51.452215],
              [-2.574301, 51.452438],
              [-2.573889, 51.452598],
              [-2.572852, 51.452994],
              [-2.57186, 51.453341],
              [-2.571252, 51.453557],
              [-2.571012, 51.45364],
              [-2.570853, 51.453697],
              [-2.5705, 51.453823],
              [-2.570369, 51.45387],
              [-2.570287, 51.453901],
              [-2.570078, 51.453977],
              [-2.569972, 51.454016],
              [-2.569803, 51.454082],
              [-2.569713, 51.454115],
              [-2.569632, 51.454144],
              [-2.569501, 51.454184],
              [-2.569137, 51.454304],
              [-2.568791, 51.454436],
              [-2.568648, 51.454505],
              [-2.568666, 51.454525],
              [-2.568674, 51.454548],
              [-2.56867, 51.454571],
              [-2.568656, 51.454593],
              [-2.568632, 51.454611],
              [-2.568601, 51.454623],
              [-2.568688, 51.454756],
              [-2.568967, 51.454972],
              [-2.569021, 51.455014],
              [-2.568715, 51.455271],
              [-2.568646, 51.455327],
              [-2.568374, 51.455559],
              [-2.568108, 51.455772],
              [-2.567496, 51.456239],
              [-2.567414, 51.456319],
              [-2.567628, 51.456492],
              [-2.567643, 51.456517],
              [-2.567649, 51.456526],
              [-2.567656, 51.456544],
              [-2.567659, 51.456553],
              [-2.567655, 51.456589],
              [-2.56766, 51.456617],
              [-2.567688, 51.456643],
              [-2.567726, 51.456662],
              [-2.567757, 51.456684],
              [-2.567783, 51.456719],
              [-2.567793, 51.456739],
              [-2.567796, 51.456747],
              [-2.56779, 51.456775],
              [-2.567829, 51.456783],
              [-2.567946, 51.456871],
              [-2.568049, 51.456948],
              [-2.568176, 51.457057],
              [-2.568213, 51.457083],
              [-2.568322, 51.45716],
              [-2.568414, 51.457216],
              [-2.56846, 51.457251],
              [-2.568445, 51.457292],
              [-2.567621, 51.457946],
              [-2.567536, 51.457943],
              [-2.567354, 51.458066],
              [-2.567316, 51.458148],
              [-2.567406, 51.458165],
              [-2.567764, 51.458232],
              [-2.568027, 51.458276],
              [-2.568076, 51.458283],
              [-2.568358, 51.458287],
              [-2.568442, 51.45829],
              [-2.568972, 51.458337],
              [-2.56919, 51.458356],
              [-2.569451, 51.458365],
              [-2.569642, 51.458363],
              [-2.569759, 51.458354],
              [-2.570133, 51.458292],
              [-2.570486, 51.458208],
              [-2.570908, 51.458114],
              [-2.571103, 51.458103],
              [-2.57128, 51.458116],
              [-2.571551, 51.458175],
              [-2.571638, 51.458205],
              [-2.571745, 51.458247],
              [-2.571982, 51.45828],
              [-2.572208, 51.458286],
              [-2.573275, 51.458227],
              [-2.573563, 51.458208],
              [-2.573671, 51.458195],
              [-2.573724, 51.458183],
              [-2.573781, 51.458175],
              [-2.573895, 51.458156],
              [-2.573943, 51.458146],
              [-2.576163, 51.457686],
              [-2.576531, 51.457615],
              [-2.576651, 51.457592],
              [-2.576817, 51.457539],
              [-2.57706, 51.4574],
              [-2.577425, 51.457222],
              [-2.577781, 51.457082],
              [-2.578035, 51.456998],
              [-2.578202, 51.456951],
              [-2.578496, 51.456874],
              [-2.578585, 51.456849],
              [-2.578781, 51.456795],
              [-2.579666, 51.456548],
              [-2.579847, 51.456493],
              [-2.579937, 51.456464],
              [-2.580159, 51.456403],
              [-2.580275, 51.456396],
              [-2.580319, 51.456385],
              [-2.580442, 51.456351],
              [-2.580625, 51.456286],
              [-2.580716, 51.456255],
              [-2.580822, 51.456205],
              [-2.580925, 51.45617],
              [-2.581018, 51.456139],
              [-2.581116, 51.456108],
              [-2.581297, 51.456054],
              [-2.581424, 51.456018],
              [-2.581562, 51.455983],
              [-2.581716, 51.455949],
              [-2.581883, 51.455913],
              [-2.582016, 51.455898],
              [-2.582766, 51.455751],
            ],
          ],
          type: "Polygon",
        },
        properties: {
          waypoints: [
            { lat: 51.455751, lon: -2.582766, snapped: true },
            { lat: 51.449653, lon: -2.57599, snapped: true },
            { lat: 51.457083, lon: -2.568213, snapped: true },
            { lat: 51.455751, lon: -2.582766, snapped: true },
          ],
        },
        type: "Feature",
      },
      addingFilter: false,
      undoLength: 0,
      redoLength: 0,
      rerender: 0,
    };
  }
</script>

<Layout>
  <div slot="left">
    {#if map}
      <MapLoader {map} bind:app />
    {/if}
    <div><button on:click={zoomToFit}>Zoom to fit</button></div>

    {#if mode.mode == "network" && app}
      <div><button on:click={setBoundaryMode}>Set boundary</button></div>
      <div><button on:click={devMode}>Quickset boundary (dev)</button></div>
    {:else if mode.mode == "set-boundary"}
      <p>Draw the boundary...</p>
    {:else if mode.mode == "neighbourhood"}
      <NeighbourhoodMode bind:mode {app} {setBoundaryMode} />
    {:else if mode.mode == "view-shortcuts"}
      <ViewShortcutsMode bind:mode {app} prevMode={mode.prevMode} {map} />
    {/if}
  </div>
  <div slot="main" style="position:relative; width: 100%; height: 100vh;">
    <MapLibre style={mapStyle} standardControls hash bind:map>
      {#if app}
        {#if mode.mode == "network"}
          <NetworkLayer {app} />
        {:else if mode.mode == "set-boundary"}
          <RouteSnapperLayer />
        {:else if mode.mode == "neighbourhood"}
          <NeighbourhoodLayer
            {map}
            {app}
            boundary={mode.boundary}
            bind:addingFilter={mode.addingFilter}
            bind:undoLength={mode.undoLength}
            bind:redoLength={mode.redoLength}
            rerender={mode.rerender}
            {offlineMode}
          />
        {:else if mode.mode == "view-shortcuts"}
          <ViewShortcutsLayer {app} />
        {/if}
      {/if}
    </MapLibre>
  </div>
</Layout>

<style>
  :global(body, button, input) {
    font-size: 26px;
  }
</style>
