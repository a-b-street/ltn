<script lang="ts">
  import { Globe } from "lucide-svelte";
  import { onMount } from "svelte";
  import { HelpButton, initTooltips } from "../common";
  import "flag-icons/css/flag-icons.min.css";
  import { appFocus } from "../stores";

  onMount(initTooltips);
</script>

<span style="display: flex; flex-direction: row; height: 80%; gap: 4px">
  <button
    class="outline icon-btn"
    on:click={() => (window.location.href = ".")}
    data-tippy-content="Global version"
    disabled={$appFocus == "global"}><Globe color="black" /></button
  >

  <button
    class="outline icon-btn"
    on:click={() => (window.location.href = "cnt.html")}
    data-tippy-content="Scotland"
    aria-label="Scotland"
    disabled={$appFocus == "cnt"}><span class="fi fi-gb-sct"></span></button
  >

  <button
    class="outline icon-btn"
    on:click={() => (window.location.href = "england.html")}
    data-tippy-content="England"
    aria-label="England"
    disabled={$appFocus == "england"}><span class="fi fi-gb-eng"></span></button
  >

  <HelpButton>
    <p>
      The LTN tool can be used anywhere in the world. Some regions have extra
      datasets. If you'd like to specialize the tool, please email Dustin at <a
        href="mailto:dabreegster@gmail.com">dabreegster@gmail.com</a
      >.
    </p>
  </HelpButton>
</span>
