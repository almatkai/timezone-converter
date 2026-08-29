import assert from "node:assert/strict";
import test, { describe, it } from "node:test";
import { DateTime } from "luxon";
import { formatZoneName, getTimezoneOffsetString, markdownForTime, convertToZone } from "../src/lib/timezone";

describe("formatZoneName", () => {
  it("replaces slashes and underscores", () => {
    assert.equal(formatZoneName("America/New_York"), "America - New York");
    assert.equal(formatZoneName("Europe/Paris"), "Europe - Paris");
    assert.equal(formatZoneName("UTC"), "UTC");
  });
});

describe("getTimezoneOffsetString", () => {
  it("returns GMT offset for known zones", () => {
    const utc = getTimezoneOffsetString("UTC");
    assert.equal(utc, "GMT+0");
    const ny = getTimezoneOffsetString("America/New_York");
    assert.ok(ny?.startsWith("GMT"));
    assert.match(ny!, /^GMT-?\d/);
  });
  it("returns undefined for undefined zone", () => {
    assert.equal(getTimezoneOffsetString(undefined), undefined);
  });
});

describe("markdownForTime", () => {
  it("produces markdown with local time title and formatted date", () => {
    const dt = DateTime.fromISO("2023-01-15T12:00:00", { zone: "UTC" });
    const md = markdownForTime(dt, false);
    assert.match(md, /Local time/);
    assert.match(md, /UTC/);
  });
  it("uses Custom time title when isCustom true", () => {
    const dt = DateTime.fromISO("2023-06-01T09:30:00", { zone: "America/New_York" });
    const md = markdownForTime(dt, true);
    assert.match(md, /Custom time/);
  });
});

describe("convertToZone", () => {
  it("converts time between zones", () => {
    const utc = DateTime.fromISO("2023-01-01T00:00:00", { zone: "UTC" });
    const ny = convertToZone(utc, "America/New_York");
    assert.equal(ny.zoneName, "America/New_York");
    // In winter, NY is UTC-5
    assert.equal(ny.hour, 19); // previous day 19:00
    assert.equal(ny.day, 31);
  });
  it("luxon handles DST", () => {
    const summerUtc = DateTime.fromISO("2023-07-01T00:00:00", { zone: "UTC" });
    const nySummer = convertToZone(summerUtc, "America/New_York");
    // NY summer is UTC-4
    assert.equal(nySummer.hour, 20);
  });
});

describe("DateTime luxon integration", () => {
  it("now in different zones differs by offset", () => {
    const now = DateTime.now();
    const utc = now.setZone("UTC");
    const tokyo = now.setZone("Asia/Tokyo");
    // Tokyo is UTC+9, so offset diff should be 9 hours modulo DST
    const diff = tokyo.offset - utc.offset;
    assert.equal(diff, 540); // 9*60 minutes
  });
});

test("package.json declares Windows+macOS and Tezbar metadata", async () => {
  const pkg = await import("../package.json", { with: { type: "json" } });
  const data = (pkg as unknown as { default: Record<string, unknown> }).default ?? (pkg as unknown as Record<string, unknown>);
  assert.deepEqual((data as { platforms: string[] }).platforms, ["macOS", "Windows"]);
  const tezbar = (data as { tezbar: { platforms: string[]; repository: string; upstream: { commit: string; path: string } } }).tezbar;
  assert.deepEqual(tezbar.platforms, ["macos", "windows"]);
  assert.equal(tezbar.repository, "https://github.com/almatkai/timezone-converter");
  assert.equal(tezbar.upstream.path, "extensions/timezone-converter");
  assert.equal(tezbar.upstream.commit, "6734d85992290066fd31a7f5bee3cca3d3545a1a");
  const commands = (data as { commands: { platforms: string[] }[] }).commands;
  assert.equal(commands.length, 1);
  for (const cmd of commands) {
    assert.deepEqual(cmd.platforms, ["macOS", "Windows"]);
  }
  const deps = (data as { dependencies: Record<string, string> }).dependencies;
  assert.ok(deps["@raycast/api"]);
  assert.ok(deps["luxon"]);
  // check normalized name (upstream was raycast-timezone-converter)
  const name = (data as { name: string }).name;
  assert.equal(name, "timezone-converter");
});
