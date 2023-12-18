<script lang="ts">
  import init, { MapModel } from "backend";
  import type { Map } from "maplibre-gl";
  import { onMount } from "svelte";
  import { Loading, OverpassSelector } from "./common";

  export let model: MapModel | undefined = undefined;
  export let map: Map;

  let example = "";
  let msg: string | null = null;
  let useLocalVite = false;

  onMount(async () => {
    await init();

    // When running locally if a vite public/ directory is set up, load from that for speed
    try {
      let resp = await fetch("/kowloon.pbf", { method: "HEAD" });
      useLocalVite = resp.ok;
      console.log("Using local cache, not od2net.org");
    } catch (err) {}
  });

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    try {
      loadModel(await fileInput.files![0].arrayBuffer());
      example = "";
    } catch (err) {
      window.alert(`Couldn't open this file: ${err}`);
    }
    msg = null;
  }

  function loadModel(buffer: ArrayBuffer) {
    msg = "Building map model from OSM input";
    console.time("load");
    model = new MapModel(new Uint8Array(buffer));
    console.timeEnd("load");
  }

  function gotXml(e: CustomEvent<string>) {
    try {
      // TODO Can we avoid turning into bytes?
      loadModel(new TextEncoder().encode(e.detail));
      example = "";
    } catch (err) {
      window.alert(`Couldn't import from Overpass: ${err}`);
    }
    msg = null;
  }

  async function loadExample(example: string) {
    if (example != "") {
      if (useLocalVite) {
        await loadFromUrl(`/${example}.pbf`);
      } else {
        await loadFromUrl(
          `https://assets.od2net.org/severance_pbfs/${example}.pbf`
        );
      }
    }
  }
  $: loadExample(example);

  async function loadFromUrl(url: string) {
    try {
      msg = `Downloading ${url}`;
      let resp = await fetch(url);
      loadModel(await resp.arrayBuffer());
    } catch (err) {
      window.alert(`Couldn't open from URL ${url}: ${err}`);
    }
    msg = null;
  }
</script>

<Loading {msg} />

<div style="border: 1px solid black; padding: 8px;">
  <div>
    <label>
      Load an osm.xml or a .pbf file:
      <input bind:this={fileInput} on:change={loadFile} type="file" />
    </label>
  </div>

  <div>
    <label>
      Or load an example:
      <select bind:value={example}>
        <option value="">Custom file loaded</option>
        <option value="akihabara">Akihabara</option>
        <option value="hanegi">Hanegi Park</option>
        <option value="harujuku">Harujuku</option>
        <option value="taipei_main_station">Taipei main station</option>
        <option value="ximending">Ximending</option>
        <option value="hong_kong">Hong Kong</option>
        <option value="kowloon">Kowloon</option>
        <option value="bristol">Bristol</option>
        <option value="elephant_castle">Elephant & Castle</option>
        <option value="westminster">Westminster</option>
        <option value="montlake">Montlake</option>
      </select>
    </label>
  </div>

  <OverpassSelector
    {map}
    on:gotXml={gotXml}
    on:loading={(e) => (msg = e.detail)}
    on:error={(e) => window.alert(e.detail)}
  />
</div>
