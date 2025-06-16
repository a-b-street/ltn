import type { FeatureCollection, MultiPolygon, Polygon } from "geojson";
import type { AppFocus } from "../stores";
import type { NeighbourhoodDefinitionFeature } from "../wasm";

export type ProjectID = ReturnType<(typeof crypto)["randomUUID"]>;
export type StudyAreaName = string | undefined;

export type UnsavedProjectFeatureCollection = FeatureCollection<
  Polygon | MultiPolygon
> & {
  // Foreign Members
  project_name: string;
  study_area_name: StudyAreaName;
  app_focus: AppFocus;
  db_schema_version?: number;
};

export type ProjectFeatureCollection = UnsavedProjectFeatureCollection & {
  db_schema_version: number;
};

/**
 * Handles migrating the local storage schema.
 */
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
    const latestSchemaVersion = 2;

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

    // Migration v0 -> v1
    //
    // This introduces schema versioning, and migrates from the old project key
    // format, which encoded project_name, study_area_name, and app_focus into
    // the key, to the new key format, which is a meaningless UUID.
    //
    // project_name, study_area_name, and app_focus are now stored in the project foreign members.
    if (this.storedSchemaVersion < 1) {
      console.log(
        `Migrating storage from version ${this.storedSchemaVersion} to ${latestSchemaVersion}`,
      );
      // Note only these two need migration. Other AppFocuses were added later.
      for (const focus of ["cnt", "global"]) {
        const appFocus = focus as AppFocus;
        const projectStorage = new ProjectStorage(
          appFocus,
          1,
          INTERNAL_METHOD_TOKEN,
        );
        for (const [studyAreaName, projects] of schemaV0_studyAreaProjects(
          appFocus,
        )) {
          for (const {
            projectKey: legacyProjectKey,
            projectName,
          } of projects) {
            let serializedProject =
              window.localStorage.getItem(legacyProjectKey);
            if (!serializedProject) {
              // I don't think this should happen. It's a bug if it does.
              console.warn(
                `Project features not found for ${legacyProjectKey}, skipping migration`,
              );
              continue;
            }
            let project = JSON.parse(serializedProject);
            project.app_focus = appFocus;
            project.study_area_name = studyAreaName;
            project.project_name = projectName;
            console.log(`migrating project ${legacyProjectKey}`);
            let id = projectStorage.createProject(project);
            console.log(
              `Successfully migrated v0->v1 project ${legacyProjectKey} -> ${projectStorage.projectKey(id)}`,
            );

            // Remove the legacy data
            window.localStorage.removeItem(legacyProjectKey);
          }
        }
      }
    }

    // Migration v1 -> v2
    //
    // Store the schemaVersion in the project foreign members.
    //
    // This is largely redundant information with Datagase.storedSchemaVersion,
    // but it makes exporting projects easier, since the schema version is now
    // stored in the project itself.
    //
    // It also serves as a nice safety point - in the case of a partially successful migration,
    // we can avoid re-migrating projects which were successfully migrated last time.
    if (this.storedSchemaVersion < 2) {
      // Note only these two need migration. Other AppFocuses were added later.
      for (const focus of ["cnt", "global"]) {
        const appFocus = focus as AppFocus;
        const projectStorage = new ProjectStorage(
          appFocus,
          2,
          INTERNAL_METHOD_TOKEN,
        );
        for (const [projectID, project] of projectStorage.projects()) {
          if (project.db_schema_version >= 2) {
            console.log(`skipping previously migrated project: ${projectID}`);
            continue;
          }
          project.db_schema_version = 2;
          projectStorage.saveProject(projectID, project);
          console.log(
            `Successfully migrated v1->v2 project: ${projectStorage.projectKey(projectID)}`,
          );
        }
      }
    }

    window.localStorage.setItem(
      this.schemaVersionKey,
      latestSchemaVersion.toString(),
    );
    console.debug(`Completed migrating database to v${latestSchemaVersion}`);
  }

  projectStorage(appFocus: AppFocus) {
    // important that this be done before projectStorage is returned
    this.ensureMigrated();
    return new ProjectStorage(
      appFocus,
      this.storedSchemaVersion,
      INTERNAL_METHOD_TOKEN,
    );
  }
}

// Simulate "file private" or "module private" methods in TypeScript
//
// You can only get an instance of ProjectStorage via Database.projectStorage(),
// to ensure that ProjectStorage is never accessed before the underlying storage has been migrated,
const INTERNAL_METHOD_TOKEN = Symbol("internal_method_token");

export class ProjectStorage {
  appFocus: AppFocus;
  dbSchemaVersion: number;

  /**
   * Don't call this method directly, use Database.projectStorage()
   */
  constructor(
    appFocus: AppFocus,
    dbSchemaVersion: number,
    internalMethodToken: symbol,
  ) {
    if (internalMethodToken !== INTERNAL_METHOD_TOKEN) {
      throw new Error(
        "ProjectStorage must be created via Database.projectStorage()",
      );
    }
    this.appFocus = appFocus;
    this.dbSchemaVersion = dbSchemaVersion;
  }

  private get collectionKey() {
    return "ltn/projects";
  }

  /**
   * This method is public, but it's only used internally and for testing.
   */
  projectKey(projectID: ProjectID): string {
    if (!projectID) {
      throw new Error("Cannot get project. ProjectID was blank/missing.");
    }
    return `${this.collectionKey}/${projectID}`;
  }

  /**
   * @returns the project name or undefined if not found
   */
  projectName(projectID: ProjectID): string | undefined {
    let project = this.project(projectID);
    return project.project_name;
  }

  projectNameAlreadyExists(projectName: string): boolean {
    for (const [_projectID, project] of this.projects()) {
      if (project.project_name == projectName) {
        return true;
      }
    }
    return false;
  }

  nextAvailableProjectName(projectName: string): string {
    if (this.projectNameAlreadyExists(projectName)) {
      let originalProjectName = projectName;
      let i = 2;
      do {
        projectName = `${originalProjectName} (${i})`;
        i++;
      } while (this.projectNameAlreadyExists(projectName));
    }
    return projectName;
  }

  nextAvailableNeighbourhoodName(projectID: ProjectID): string {
    let basename = "My neighbourhood";

    let project = this.project(projectID);
    let neighbourhoods = project.features.filter(
      (f) => f.properties?.kind == "boundary",
    ) as Array<NeighbourhoodDefinitionFeature>;

    let neighbourhoodName = basename;
    let i = 1;
    while (
      neighbourhoods.some((n) => n.properties?.name == neighbourhoodName)
    ) {
      i++;
      neighbourhoodName = `${basename} #${i}`;
    }
    return neighbourhoodName;
  }

  /**
   * @throws if the project name already exists
   */
  createEmptyProject(
    projectName: string,
    studyAreaName: StudyAreaName,
  ): ProjectID {
    const project: UnsavedProjectFeatureCollection = {
      type: "FeatureCollection" as const,
      features: [],
      study_area_name: studyAreaName,
      project_name: projectName,
      app_focus: this.appFocus,
    };
    return this.createProject(project);
  }

  /**
   * @throws if the project name already exists
   */
  createProject(project: UnsavedProjectFeatureCollection): ProjectID {
    if (this.projectNameAlreadyExists(project.project_name)) {
      throw new Error(`Project name already taken: ${project.project_name}`);
    }
    if (!project.db_schema_version) {
      project.db_schema_version = this.dbSchemaVersion;
    }
    let projectID = crypto.randomUUID();
    this.saveProject(projectID, project as ProjectFeatureCollection);
    return projectID;
  }

  /**
   * @returns An array of projects, grouped by study area name.
   */
  studyAreaProjects(): Array<
    [StudyAreaName, Array<{ projectID: ProjectID; projectName: string }>]
  > {
    let projectsByStudyArea = new Map();

    for (const [projectID, project] of this.projects()) {
      const studyAreaName = project.study_area_name;

      if (!projectsByStudyArea.has(studyAreaName)) {
        projectsByStudyArea.set(studyAreaName, []);
      }

      projectsByStudyArea.get(studyAreaName)!.push({
        projectID: projectID as ProjectID,
        projectName: project.project_name,
      });
    }

    // Sort the projects by name
    for (const [_studyAreaName, projects] of projectsByStudyArea.entries()) {
      projects.sort((a: { projectName: string }, b: { projectName: string }) =>
        a.projectName.localeCompare(b.projectName),
      );
    }

    return Array.from(projectsByStudyArea.entries()).sort((a, b) => {
      if (a[0] == undefined) return 1;
      if (b[0] == undefined) return -1;
      return a[0].localeCompare(b[0]);
    });
  }

  projectIDFromKey(key: string): ProjectID | undefined {
    let match = key.match(/ltn\/projects\/([a-z0-9-]+)/);
    if (!match) {
      return undefined;
    }
    let projectID = match[1];
    if (!projectID) {
      return undefined;
    }
    return projectID as ProjectID;
  }

  projects(): Array<[ProjectID, ProjectFeatureCollection]> {
    return this.allProjectKeys()
      .map((key) => {
        let projectID = this.projectIDFromKey(key);
        if (!projectID) {
          throw new Error(
            `Cannot get project: no projectID parsed from ${key}`,
          );
        }
        let projectString = window.localStorage.getItem(key);
        if (!projectString) {
          throw new Error(`Cannot get project: no project found for ${key}`);
        }
        let project = JSON.parse(projectString);
        if (project.app_focus != this.appFocus) {
          return null;
        }
        let entry: [ProjectID, ProjectFeatureCollection] = [projectID, project];
        return entry;
      })
      .filter((entry) => entry !== null);
  }

  private allProjectKeys(): string[] {
    const prefix = this.collectionKey;
    let keys = [];
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (key && key.startsWith(prefix)) {
        keys.push(key);
      }
    }
    return keys;
  }

  saveProject(projectID: ProjectID, project: ProjectFeatureCollection) {
    let key = this.projectKey(projectID);

    if (!project.study_area_name) {
      // unify null, undefined, etc.
      delete project.study_area_name;
    }

    window.localStorage.setItem(key, JSON.stringify(project));
  }

  removeProject(projectID: ProjectID) {
    let key = this.projectKey(projectID);
    window.localStorage.removeItem(key);
  }

  renameProject(projectID: ProjectID, newName: string) {
    let project = this.project(projectID);
    if (!project) {
      throw new Error(
        `Cannot rename project: no existing project found for ${projectID}`,
      );
    }
    if (project.project_name == newName) {
      console.debug("Ignoring rename, since it's the same name.");
      return;
    }
    if (this.projectNameAlreadyExists(newName)) {
      throw new Error(
        `Project name ${newName} already taken. Choose a different name.`,
      );
    }
    project.project_name = newName;
    this.saveProject(projectID, project);
  }

  copyProject(projectID: ProjectID, newName: string): ProjectID {
    let project = this.project(projectID);
    if (!project) {
      throw new Error(
        `Cannot copy project: no existing project found for ${projectID}`,
      );
    }
    if (this.projectNameAlreadyExists(newName)) {
      throw new Error(
        `Project name ${newName} already taken. Choose a different name.`,
      );
    }
    let newProjectID = crypto.randomUUID();
    project.project_name = newName;
    this.saveProject(newProjectID, project);
    return newProjectID;
  }

  project(projectID: ProjectID): ProjectFeatureCollection {
    let key = this.projectKey(projectID);
    let projectString = window.localStorage.getItem(key);
    if (!projectString) {
      throw new Error(`Cannot get project: no project found for ${projectID}`);
    }
    let project = JSON.parse(projectString);
    console.assert!(
      project.study_area_name || project.app_focus != "cnt",
      "missing study area name for cnt project",
    );
    console.assert!(project.project_name, "missing project name");
    return project;
  }
}

// Returns a list, grouped and sorted by the optional studyAreaId, with
// custom cases at the end
function schemaV0_studyAreaProjects(
  appFocus: AppFocus,
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
      let parts = projectKey.split("ltn_");
      let projectName = parts[parts.length - 1];
      let studyAreaName = "";
      try {
        let gj = JSON.parse(window.localStorage.getItem(projectKey)!);
        studyAreaName = gj.study_area_name;
      } catch (err) {
        console.log(`error loading global project: ${projectKey}`, err);
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
