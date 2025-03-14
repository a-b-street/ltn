<script lang="ts">
  import { stripPrefix, stripSuffix } from "../common";
  import { projectStorage } from "../stores";
  import { loadProject } from "./loader";

  export let loading: string;

  let fileInput: HTMLInputElement;

  async function loadFile(e: Event) {
    let filename = fileInput.files![0].name;
    loading = `Loading from file ${filename}`;

    let contents = await fileInput.files![0].text();
    let gj = JSON.parse(contents);

    let projectName: string;

    let studyAreaName: string = "";

    // Legacy loading logic, this can go away at some point if we want to drop support for old saved file formats.

    // Is this a CNT project or a global one?
    if (gj.study_area_name && gj.study_area_name.startsWith("LAD_")) {
      let kind = "ltn_cnt";
      // Parse the project name from the filename, best effort. The user may
      // have renamed the file.
      projectName = stripSuffix(
        stripPrefix(filename, `${kind}_${gj.study_area_name}_`),
        ".geojson",
      );
    } else {
      projectName = stripSuffix(stripPrefix(filename, "ltn_"), ".geojson");
    }

    if (gj.study_area_name) {
      studyAreaName = gj.study_area_name;
    }

    // modern loading logic
    if (gj.projectSummary?.projectName) {
      projectName = gj.projectSummary.projectName;
    }
    if (gj.projectSummary?.studyAreaName) {
      studyAreaName = gj.projectSummary.studyAreaName;
    }

    if ($projectStorage.projectNameAlreadyExists(projectName)) {
      let i = 2;
      do {
        projectName = `${projectName} (${i})`;
        i++;
      } while ($projectStorage.projectNameAlreadyExists(projectName));
    }

    let projectID = $projectStorage.createNewProject(
      projectName,
      studyAreaName,
    );
    $projectStorage.saveProject(projectID, gj);
    await loadProject(projectID);
    loading = "";
  }
</script>

<label style="margin-top: 16px;">
  <strong>Load a project from a file</strong>
  <input bind:this={fileInput} on:change={loadFile} type="file" />
</label>
