import { describe, expect, it } from "vitest";
import { searchFolders } from "./folder-search";

describe("folder search", () => {
  const folders = ["/work/gasd-core", "/work/website", "/home/Documents"];
  it("keeps all folders for an empty search", () => {
    expect(searchFolders(folders, "  ")).toEqual(folders);
  });
  it("matches non-contiguous characters without case sensitivity", () => {
    expect(searchFolders(folders, "GSC")).toEqual(["/work/gasd-core"]);
  });
  it("ranks contiguous matches first and excludes non-matches", () => {
    expect(searchFolders(["/w/a-b-c", "/w/abc", "/w/xyz"], "abc"))
      .toEqual(["/w/abc", "/w/a-b-c"]);
  });
});
