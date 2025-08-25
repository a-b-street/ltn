<script lang="ts">
  import { stopPropagation } from 'svelte/legacy';

  // This launches a modal when clicked. The user should put the modal contents
  // as the child snippet beneath this component.
  import { CircleHelp } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import { Modal } from "svelte-utils";

  let show = $state(false);
  interface Props {
    color?: string;
    children: Snippet;
  }

  let { color = "black", children }: Props = $props();
</script>

<button
  class="icon-btn help"
  title="Help"
  onclick={stopPropagation(() => (show = true))}
>
  <CircleHelp {color} />
</button>

<div class="pico">
  <Modal bind:show>
    <article style="max-height: 80vh; max-width: 80vw; overflow: auto">
      <h2>Help</h2>
      {@render children()}
    </article>
  </Modal>
</div>

<style>
  button.help {
    background: none;
    border: none;
  }

  /* 
  special considerations so that the help button doesn't expand the
  height of the breadcrumbs
  */
  :global(.pico nav[aria-label="breadcrumb"] ul > li) button.help {
    height: 30px;
    padding: 2px;
    width: auto;
  }
</style>
