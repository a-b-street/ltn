import type { FeatureCollection } from "geojson";

export type ProjectID = ReturnType<(typeof crypto)["randomUUID"]>;
export type StudyAreaName = string | undefined;
export type ProjectSummary = {
  projectName: string;
  studyAreaName: StudyAreaName;
  appFocus: "cnt" | "global";
};

type ProjectIndex = {
  projects: Record<ProjectID, ProjectSummary>;
};

export class Database {
  private rootKey = "ltn";
  private get schemaVersionKey() {
    return `${this.rootKey}/_meta/schema-version`;
  }

  private get storedSchemaVersion(): number {
    let schemaVersion: number;
    let schemaVersionString = window.localStorage.getItem(
      this.schemaVersionKey,
    );
    if (schemaVersionString) {
      schemaVersion = parseInt(schemaVersionString);
      if (Number.isNaN(schemaVersion)) {
        schemaVersion = 0;
      }
    } else {
      schemaVersion = 0;
    }
    return schemaVersion;
  }

  ensureMigrated() {
    // increment this and add a new logic block if you need to add a migration
    const latestSchemaVersion = 1;

    if (localStorage.length == 0) {
      console.log("localStorage is empty — nothing to migrate.");
      window.localStorage.setItem(
        this.schemaVersionKey,
        latestSchemaVersion.toString(),
      );
      return;
    }

    if (this.storedSchemaVersion == latestSchemaVersion) {
      console.debug("Schema is already up to date.");
      return;
    }

    if (this.storedSchemaVersion < 1) {
      console.log(
        `Migrating storage from version ${this.storedSchemaVersion} to ${latestSchemaVersion}`,
      );
      for (const focus of ["cnt", "global"]) {
        const appFocus = focus as "cnt" | "global";
        const projectStorage = new ProjectStorage(
          appFocus,
          INTERNAL_METHOD_TOKEN,
        );
        // seemingly none of this is running - is listProjects empty?
        for (const [studyAreaName, projects] of schemaV0_listProjects(
          appFocus,
        )) {
          for (const { projectKey, projectName } of projects) {
            let projectFeatures = window.localStorage.getItem(projectKey);
            if (!projectFeatures) {
              console.warn(
                `Project features not found for ${projectKey}, skipping migration`,
              );
              continue;
            }

            let id = projectStorage.createNewProject(
              projectName,
              studyAreaName,
            );
            console.log(
              `migrating project ${projectKey} to ${projectStorage.projectKey(id)}`,
            );
            projectStorage.saveProject(id, JSON.parse(projectFeatures));

            // Remove the old data
            window.localStorage.removeItem(projectKey);
          }
        }
      }
    }
    window.localStorage.setItem(
      this.schemaVersionKey,
      latestSchemaVersion.toString(),
    );
  }

  projectStorage(appFocus: "cnt" | "global") {
    // important that this be done before projectStorage is returned
    this.ensureMigrated();
    return new ProjectStorage(appFocus, INTERNAL_METHOD_TOKEN);
  }
}

// Simulate "file private" or "module private" methods in TypeScript
//
// I want to ensure that ProjectStorage is only created through Storage.projectStorage
const INTERNAL_METHOD_TOKEN = Symbol("internal_method_token");

export class ProjectStorage {
  appFocus: "global" | "cnt";
  index: ProjectIndex;

  /**
   * Don't call this method directly, use Database.projectStorage()
   */
  constructor(appFocus: "global" | "cnt", internalMethodToken: symbol) {
    if (internalMethodToken !== INTERNAL_METHOD_TOKEN) {
      throw new Error(
        "ProjectStorage must be created via Storage.projectStorage()",
      );
    }
    this.appFocus = appFocus;
    this.index = {
      projects: {},
    };

    window.addEventListener("storage", (e) => {
      // Storage changed in another tab. Re-load index so that we're up to date.
      // Otherwise, our `saves` could clobber data.
      //
      // Still, without transactions, this is potentially racey. e.g. if two
      // tabs save at the exact same time.
      if (e.key == this.indexKey) {
        console.log("reloading index after remote write.");
        this.loadIndex();
      } else {
        console.log("ignoring non-index storage event", e);
      }
    });
    this.loadIndex();
    this.loadIndex();
  }

  private get collectionKey() {
    return "ltn/projects";
  }

  /**
   * This method is public, but it's only used internally and for testing.
   */
  projectKey(projectID: ProjectID): string {
    if (!projectID) {
      throw new Error("Cannot get project key: no ID provided");
    }
    return `${this.collectionKey}/by-id/${projectID}`;
  }

  private get indexKey(): string {
    return `${this.collectionKey}/index`;
  }

  // Normally you shouldn't need to call this, but the unit tests
  // edit localStorage directly.
  public reloadIndexForTesting() {
    this.loadIndex();
  }

  private loadIndex() {
    let indexString = window.localStorage.getItem(this.indexKey);
    if (indexString) {
      this.index = JSON.parse(indexString);
    } else {
      this.index = {
        projects: {},
      };
    }
  }

  private saveIndex() {
    window.localStorage.setItem(this.indexKey, JSON.stringify(this.index));
  }

  /**
   * @returns the project name or undefined if not found
   */
  projectName(projectID: ProjectID): string | undefined {
    let projectSummary = this.index.projects[projectID];
    if (!projectSummary) {
      console.warn(`Project summary for ${projectID} not found`);
      return undefined;
    }
    return projectSummary.projectName;
  }

  projectNameAlreadyExists(projectName: string): boolean {
    return !!Object.values(this.index.projects).find(
      (p) => p.projectName == projectName,
    );
  }

  /**
   * @throws if the project name already exists
   */
  createNewProject(
    projectName: string,
    studyAreaName: StudyAreaName,
  ): ProjectID {
    if (this.projectNameAlreadyExists(projectName)) {
      throw new Error(`Project name already taken: ${projectName}`);
    }

    const projectSummary: ProjectSummary = {
      projectName,
      studyAreaName,
      appFocus: this.appFocus,
    };

    let projectID = crypto.randomUUID();
    const project = {
      type: "FeatureCollection" as const,
      features: [],
    };
    this.index.projects[projectID] = projectSummary;
    this.saveProject(projectID, project);
    this.saveIndex();
    return projectID;
  }

  /**
   * @returns An array of projects, grouped by study area name.
   */
  listProjects(): Array<
    [StudyAreaName, Array<{ projectID: ProjectID; projectName: string }>]
  > {
    let projectsByStudyArea = new Map();

    for (const [projectID, project] of Object.entries(this.index.projects)) {
      if (project.appFocus != this.appFocus) {
        continue;
      }
      const studyAreaName = project.studyAreaName;

      if (!projectsByStudyArea.has(studyAreaName)) {
        projectsByStudyArea.set(studyAreaName, []);
      }

      projectsByStudyArea.get(studyAreaName)!.push({
        projectID: projectID as ProjectID,
        projectName: project.projectName,
      });
    }

    return Array.from(projectsByStudyArea.entries()).sort((a, b) => {
      if (a[0] == undefined) return 1;
      if (b[0] == undefined) return -1;
      return a[0].localeCompare(b[0]);
    });
  }

  saveProject(projectID: ProjectID, project: FeatureCollection) {
    if (!projectID) {
      throw new Error("Cannot save project: no ID provided");
    }
    console.assert(
      this.index.projects[projectID],
      `no index entry for projectID: ${projectID}`,
    );
    let key = this.projectKey(projectID);
    window.localStorage.setItem(key, JSON.stringify(project));
  }

  removeProject(projectID: ProjectID) {
    console.assert(this.index.projects[projectID]);
    delete this.index.projects[projectID];
    this.saveIndex();

    let key = this.projectKey(projectID);
    window.localStorage.removeItem(key);
  }

  renameProject(projectID: ProjectID, newName: string) {
    let projectSummary = this.index.projects[projectID];
    if (!projectSummary) {
      throw new Error(`Project summary for ${projectID} not found`);
    }
    if (projectSummary.projectName == newName) {
      console.warn(`Project ${projectID} already has name ${newName}`);
      return;
    }
    if (this.projectNameAlreadyExists(newName)) {
      throw new Error(
        `Project name ${newName} already taken. Choose a different name.`,
      );
    }
    this.index.projects[projectID].projectName = newName;
    this.saveIndex();
  }

  getProject(projectID: ProjectID): {
    projectSummary: ProjectSummary;
    features: FeatureCollection;
  } {
    let key = this.projectKey(projectID);
    let projectSummary = this.index.projects[projectID];
    if (!projectSummary) {
      throw new Error(`Cannot get project: no summary found for ${projectID}`);
    }
    let projectString = window.localStorage.getItem(key);
    if (!projectString) {
      throw new Error(`Cannot get project: no project found for ${projectID}`);
    }
    let features = JSON.parse(projectString);
    return { projectSummary, features };
  }
}

// Returns a list, grouped and sorted by the optional studyAreaId, with
// custom cases at the end
function schemaV0_listProjects(
  appFocus: "cnt" | "global",
): Array<[StudyAreaName, { projectKey: string; projectName: string }[]]> {
  let studyAreas = new Map();
  let custom = [];
  for (let i = 0; i < window.localStorage.length; i++) {
    let projectKey = window.localStorage.key(i)!;
    if (projectKey.startsWith("ltn_cnt/")) {
      if (appFocus != "cnt") {
        continue;
      }
      try {
        let [_, studyAreaId, projectName] = projectKey.split("/");
        if (!studyAreas.has(studyAreaId)) {
          studyAreas.set(studyAreaId, []);
        }
        studyAreas.get(studyAreaId)!.push({ projectKey, projectName });
      } catch (err) {
        console.log(`error loading cnt project: ${projectKey}`, err);
      }
    } else if (projectKey.startsWith("ltn_")) {
      if (appFocus != "global") {
        continue;
      }
      let projectName = projectKey.split("ltn_")[1];
      let studyAreaName = "";
      try {
        let gj = JSON.parse(window.localStorage.getItem(projectKey)!);
        studyAreaName = gj.study_area_name;
      } catch (err) {
        console.log(`error loading cnt project: ${projectKey}`, err);
      }
      if (studyAreaName && studyAreaName.length > 0) {
        if (!studyAreas.has(studyAreaName)) {
          studyAreas.set(studyAreaName, []);
        }
        studyAreas.get(studyAreaName)!.push({ projectKey, projectName });
      } else {
        custom.push({ projectKey, projectName });
      }
    }
  }

  let out = [...studyAreas.entries()];
  out.sort((a, b) => a[0].localeCompare(b[0]));
  if (custom.length > 0) {
    out.push([undefined, custom]);
  }
  return out;
}
