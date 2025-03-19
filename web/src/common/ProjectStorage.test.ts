import type { FeatureCollection } from "geojson";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  Database,
  ProjectStorage,
  type ProjectFeatureCollection,
} from "./ProjectStorage";

let projectStorage: ProjectStorage;

// Helper function to mock localStorage with predefined data
function mockLocalStorage(mockData = {}) {
  const storage: Record<string, string> = {};

  Object.entries(mockData).forEach(([key, value]) => {
    storage[key] = JSON.stringify(value);
  });

  Object.defineProperty(window, "localStorage", {
    // mock localStorage API's that we use
    value: {
      getItem: vi.fn((key) => storage[key] || null),
      setItem: vi.fn((key, value) => {
        storage[key] = value;
      }),
      removeItem: vi.fn((key) => {
        delete storage[key];
      }),
      clear: vi.fn(),
      key: vi.fn((index) => Object.keys(storage)[index] || null),
      get length() {
        return Object.keys(storage).length;
      },
    },
    writable: true,
  });
  projectStorage = new Database().projectStorage("cnt");

  return { storage, localStorage: window.localStorage };
}

beforeEach(() => {
  vi.resetAllMocks();
  mockLocalStorage();
});

afterEach(() => {
  vi.resetAllMocks();
});

describe("ProjectStorage", () => {
  describe("createEmptyProject", () => {
    it("should save project data and update index if it's a known project", () => {
      let { storage } = mockLocalStorage({});
      let id = projectStorage.createEmptyProject("Project Name", "TestArea");
      let key = projectStorage.projectKey(id);
      let originalProject: ProjectFeatureCollection = JSON.parse(storage[key]);
      expect(originalProject).toBeDefined();
      expect(originalProject.features).toHaveLength(0);
      originalProject.features.push("FakeFeature" as any);
      projectStorage.saveProject(id, originalProject);

      let reloadedProject: FeatureCollection = JSON.parse(storage[key]);
      expect(reloadedProject.features).toHaveLength(1);
    });
  });

  describe("studyAreas", () => {
    it("should list projects from the index by study area", () => {
      projectStorage.createEmptyProject("Project 1", "Edinburgh");
      projectStorage.createEmptyProject("Project 2", "Glasgow");
      projectStorage.createEmptyProject("Project 3", "Edinburgh");

      const result = projectStorage.studyAreaProjects();
      console.log(result);
      expect(result).toHaveLength(2); // Two study areas

      let edinburgh = result[0];
      expect(edinburgh[0]).toBe("Edinburgh");
      let edinburghProjects = edinburgh[1];
      expect(edinburghProjects).toHaveLength(2);
      expect(edinburghProjects[0].projectName).toBe("Project 1");
      expect(edinburghProjects[1].projectName).toBe("Project 3");

      let glasgow = result[1];
      expect(glasgow[0]).toBe("Glasgow");
      let glasgowProjects = glasgow[1];
      expect(glasgowProjects).toHaveLength(1);
      expect(glasgowProjects[0].projectName).toBe("Project 2");
    });

    it("should return empty array when index is empty", () => {
      const result = projectStorage.studyAreaProjects();
      expect(result).toHaveLength(0);
    });
  });

  describe("removeProject", () => {
    it("should remove project and update index", () => {
      let { storage } = mockLocalStorage({});
      let id = projectStorage.createEmptyProject("Project Name", "TestArea");
      let key = projectStorage.projectKey(id);
      expect(storage[key]).toBeDefined();

      projectStorage.removeProject(id);
      expect(storage[key]).not.toBeDefined();
    });

    it("should not throw error when project doesn't exist", () => {
      expect(() =>
        projectStorage.removeProject("ce-nest-pas-un-uuid"),
      ).not.toThrow();
    });
  });

  describe("renameProject", () => {
    it("should rename a project by updating the projectSummary", () => {
      let id = projectStorage.createEmptyProject("Original Name", "TestArea");
      expect(projectStorage.projectName(id)).toBe("Original Name");

      projectStorage.renameProject(id, "New Name");
      expect(projectStorage.projectName(id)).toBe("New Name");
    });

    it("should throw error when project doesn't exist", () => {
      expect(() =>
        projectStorage.renameProject("ce-nest-pas-un-uuid", "New Name"),
      ).toThrow("Cannot get project: no project found for ce-nest-pas-un-uuid");
    });
  });

  describe("projectNameAlreadyExists", () => {
    it("should return true if project name already exists", () => {
      projectStorage.createEmptyProject("Existing Project", "TestArea");
      expect(projectStorage.projectNameAlreadyExists("Existing Project")).toBe(
        true,
      );
    });

    it("should return false if project name does not exist", () => {
      expect(
        projectStorage.projectNameAlreadyExists("Nonexistent Project"),
      ).toBe(false);
    });
  });
});
