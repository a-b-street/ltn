<script lang="ts">
  import { Modal } from "svelte-utils";
  import { appFocus, showAbout } from "./stores";
</script>

<Modal bind:show={$showAbout}>
  {#if $appFocus == "cnt"}
    <h1>The Connected Neighbourhoods Tool</h1>
  {:else}
    <h1>The Low-Traffic Neighbourhood (LTN) tool, v2</h1>
  {/if}

  <p>
    This is the second version of the A/B Street LTN tool, <a
      href="https://github.com/a-b-street/ltn"
      target="_blank">an open source project</a
    >. This version, called the Connected Neighbourhoods Tool, has been funded
    by
    <a href="https://www.transport.gov.scot/" target="_blank"
      >Transport Scotland</a
    >
    and designed by
    <a href="https://www.sustrans.org.uk" target="_blank">Sustrans Scotland</a>.
    The team includes:
  </p>
  <ul>
    <li>
      Dustin Carlino, lead developer and director of <a
        href="https://find-and-update.company-information.service.gov.uk/company/16217327"
        target="_blank">A/B Street Ltd</a
      >
    </li>
    <li>
      <a href="https://github.com/michaelkirk/" target="_blank">Michael Kirk</a
      >, software developer
    </li>
    <li>
      Sustrans Scotland
      <ul>
        <li>
          <a href="https://www.linkedin.com/in/congying-hu/" target="_blank"
            >Congying (Cici) Hu</a
          >, mobility planner
        </li>
        <li>
          <a
            href="https://www.linkedin.com/in/angus-calder-a69543206"
            target="_blank">Angus Calder</a
          >, senior mobility planner
        </li>
        <li>Michael Naysmith, senior mobility planner (project manager)</li>
        <li>
          <a
            href="https://www.linkedin.com/in/matthew-davis-6b3a8124/"
            target="_blank">Matthew Davis</a
          >, principal mobility planner
        </li>
      </ul>
    </li>
  </ul>

  <p>
    This second version of the tool is an evolution from the original A/B Street
    tool, with <a
      href="https://a-b-street.github.io/docs/software/ltn/index.html#credits"
      target="_blank">many people</a
    > to thank there. Of the dozens of users giving excellent feedback and shaping
    the tool’s development, particular thanks to:
  </p>

  <ul>
    <li>
      <a href="https://github.com/XioNoX" target="_blank">Arzhel Younsi</a>
    </li>
    <li>
      <a href="https://github.com/rouelibre1" target="_blank">Rouelibre1</a>
    </li>
  </ul>

  <p>This tool would not be possible without:</p>
  <ul>
    <li>
      <a href="https://www.openstreetmap.org/about" target="_blank"
        >OpenStreetMap contributors</a
      >
    </li>
    <li><a href="https://www.maptiler.com/" target="_blank">Maptiler</a></li>
    <li><a href="https://georust.org/" target="_blank">GeoRust</a></li>
    <li>
      <a href="https://lucide.dev/" target="_blank">Lucide icons</a>
    </li>
    <li>And countless other open source libraries</li>
  </ul>

  <p>Some road signs images © Crown copyright.</p>
</Modal>
