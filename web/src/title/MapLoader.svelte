<script lang="ts">
  import { LTN } from "backend";
  import { onMount } from "svelte";
  import { Loading, OverpassSelector } from "../common";
  import { app, example, map } from "../stores";

  let msg: string | null = null;
  let useLocalVite = false;

  onMount(async () => {
    // When running locally if a vite public/ directory is set up, load from that for speed
    try {
      let resp = await fetch("/osm/kowloon.pbf", { method: "HEAD" });
      useLocalVite = resp.ok;
      console.log("Using local cache, not od2net.org");

      // For quicker dev
      //$example = "bristol";
    } catch (err) {}
  });

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    try {
      loadMap(await fileInput.files![0].arrayBuffer());
      $example = "";
    } catch (err) {
      window.alert(`Couldn't open this file: ${err}`);
    }
    msg = null;
  }

  function loadMap(buffer: ArrayBuffer) {
    msg = "Building map model from OSM input";
    console.time("load");
    $app = new LTN(new Uint8Array(buffer));
    console.timeEnd("load");
  }

  function gotXml(e: CustomEvent<string>) {
    try {
      // TODO Can we avoid turning into bytes?
      loadMap(new TextEncoder().encode(e.detail));
      $example = "";
    } catch (err) {
      window.alert(`Couldn't import from Overpass: ${err}`);
    }
    msg = null;
  }

  async function loadExample(ex: string) {
    if (ex != "") {
      if (useLocalVite) {
        await loadFromUrl(`/osm/${ex}.pbf`);
      } else {
        await loadFromUrl(`https://assets.od2net.org/severance_pbfs/${ex}.pbf`);
      }
    }
  }
  $: loadExample($example);

  async function loadFromUrl(url: string) {
    try {
      msg = `Downloading ${url}`;
      let resp = await fetch(url);
      loadMap(await resp.arrayBuffer());
    } catch (err) {
      window.alert(`Couldn't open from URL ${url}: ${err}`);
    }
    msg = null;
  }
</script>

<Loading {msg} />

<div>
  <label>
    Load an example:
    <select bind:value={$example}>
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
      <option value="strasbourg">Strasbourg</option>
    </select>
  </label>
</div>

<div>
  <label>
    Load an osm.xml or a .pbf file:
    <input bind:this={fileInput} on:change={loadFile} type="file" />
  </label>
</div>

<OverpassSelector
  map={$map}
  on:gotXml={gotXml}
  on:loading={(e) => (msg = e.detail)}
  on:error={(e) => window.alert(e.detail)}
/>
