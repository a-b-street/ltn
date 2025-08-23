<script lang="ts">
  import { Modal } from "svelte-utils";
  import { ModalFilterType } from "../common/ModalFilterType";
  import { currentFilterType } from "../stores";

  export let show: boolean;
  $: currentFilter = ModalFilterType.allTypes.find(
    (x) => x.filterType == $currentFilterType,
  )!;
</script>

<Modal bind:show
  ><article style="max-height: 80vh; max-width: 80vw; overflow: auto">
    <h3>Choose a modal filter to place on streets</h3>
    <table>
      <tbody>
        <tr>
          <td>
            {#each ModalFilterType.allTypes as filter}
              <button
                class="outline"
                style="width: 100%"
                disabled={$currentFilterType == filter.filterType}
                onclick={() => ($currentFilterType = filter.filterType)}
              >
                <img src={filter.iconURL} width="80" alt={filter.label} />
                <br />
                {filter.label}
              </button>
            {/each}
          </td>
          <td>
            <img
              src={currentFilter.largeImageURL}
              height="300"
              alt={currentFilter.label}
            />
            <p>{currentFilter.description}</p>
          </td>
        </tr>
      </tbody>
    </table>
    <center>
      <button onclick={() => (show = false)}>Confirm</button>
    </center>
  </article></Modal
>
