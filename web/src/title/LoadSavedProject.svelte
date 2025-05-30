<script lang="ts">
  import { stripPrefix, stripSuffix } from "../common";
  import type { StudyAreaName } from "../common/ProjectStorage";
  import { database, mode, type AppFocus } from "../stores";
  import { loadProject } from "./loader";

  let fileInput: HTMLInputElement;

  async function loadFile(e: Event) {
    let filename = fileInput.files![0].name;

    let contents = await fileInput.files![0].text();
    let gj = JSON.parse(contents);

    let appFocus: AppFocus;
    let studyAreaName: StudyAreaName;
    let projectName: string;

    // modern (v1) save files will have these fields set
    // legacy (v0) save files will not (except study_area_name, which is set *excepting* "global" custom (overpass) areas)
    if (gj.app_focus) {
      console.assert!(
        gj.app_focus == "global" ||
          gj.app_focus == "cnt" ||
          gj.app_focus == "england",
      );
      appFocus = gj.app_focus;
    }
    if (gj.study_area_name) {
      studyAreaName = gj.study_area_name;
    }
    if (gj.project_name) {
      projectName = gj.project_name;
    }

    if (!gj.app_focus || !gj.project_name) {
      // This is a legacy savefile (from before schema v0)
      //
      // This loading logic could go away at some point if we want to drop
      // support for old saved file formats.
      //
      // Is this a CNT project or a global one?
      if (gj.study_area_name && gj.study_area_name.startsWith("LAD_")) {
        // Note england projects also have boundaries starting with LAD_, but
        // they were created after the legacy savefile
        appFocus = "cnt";
        // Parse the project name from the filename, best effort. The user may
        // have renamed the file.
        projectName = stripSuffix(
          stripPrefix(filename, `ltn_cnt_${gj.study_area_name}_`),
          ".geojson",
        );
      } else {
        appFocus = "global";
        projectName = stripSuffix(stripPrefix(filename, "ltn_"), ".geojson");
      }
    }
    // Verified non-null by the above if statement
    projectName = projectName!;
    appFocus = appFocus!;

    let projectStorage = database.projectStorage(appFocus);
    projectName = projectStorage.nextAvailableProjectName(projectName)!;

    gj.project_name = projectName;
    gj.app_focus = appFocus!;
    gj.study_area_name = studyAreaName;

    let projectID = projectStorage.createProject(gj);
    await loadProject(projectID);
    $mode = { mode: "pick-neighbourhood" };
  }
</script>

<label style="margin-top: 16px;">
  <strong>Load a project from a file</strong>
  <input bind:this={fileInput} on:change={loadFile} type="file" />
</label>
