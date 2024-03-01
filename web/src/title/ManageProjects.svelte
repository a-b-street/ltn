<script lang="ts">
  let projectList = getProjectList();

  function getProjectList(): string[] {
    let list = [];
    for (let i = 0; i < window.localStorage.length; i++) {
      let key = window.localStorage.key(i)!;
      if (key.startsWith("ltn_")) {
        list.push(key);
      }
    }
    list.sort();
    return list;
  }
</script>

Load a saved project:
<ul>
  {#each projectList as project}
    <li>{project}</li>
  {/each}
</ul>
