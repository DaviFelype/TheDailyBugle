// T15–T20 and S08–S10, S12 of docs/test-strategy/001-walking-skeleton.md.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import vm from "node:vm";

import { loadModel } from "./load-model.mjs";

const model = loadModel();

function fixture(name) {
  return readFileSync(new URL(`../fixtures/feed-v1/${name}`, import.meta.url), "utf8");
}

// Objects made inside the vm have another realm's prototypes; compare data only.
function plain(value) {
  return JSON.parse(JSON.stringify(value));
}

function feedWith(items) {
  return JSON.stringify({ schemaVersion: 1, items });
}

const ITEM = {
  id: "x:1",
  category: "news",
  title: "Title",
  url: "https://example.com/1",
  source: "Example",
};

test("a valid feed is ok and keeps every item", () => {
  const state = model.parseFeed(fixture("valid.json"), 0);

  assert.equal(state.status, "ok");
  assert.deepEqual(
    plain(state.items).map((item) => item.category),
    ["vulnerability", "ransomware", "news"],
  );
});

test("a failed run, empty output or non-object JSON is an error, never an exception", () => {
  const cases = [
    [fixture("valid.json"), 1],
    [fixture("valid.json"), null],
    ["", 0],
    ["{", 0],
    ["null", 0],
    ["[]", 0],
    ['"text"', 0],
    ['{"schemaVersion":1}', 0],
    ['{"schemaVersion":"1","items":[]}', 0],
    [undefined, 0],
  ];
  for (const [stdout, exitCode] of cases) {
    assert.equal(model.parseFeed(stdout, exitCode).status, "error", `input: ${stdout} / ${exitCode}`);
  }
});

test("another schema version is incompatible", () => {
  assert.equal(model.parseFeed(fixture("schema-version-2.json"), 0).status, "incompatible");
});

test("unknown fields are ignored and not passed on", () => {
  const state = model.parseFeed(fixture("extra-field.json"), 0);

  assert.equal(state.status, "ok");
  assert.equal(state.items.length, 1);
  assert.equal(state.items[0].severity, undefined);
});

test("a failed run keeps the last good items", () => {
  const good = model.nextState(model.initialState(), fixture("valid.json"), 0);
  const failed = model.nextState(good, "", 1);

  assert.equal(failed.status, "error");
  assert.equal(failed.items.length, 3);
});

test("a good run replaces the previous state", () => {
  const failed = model.nextState(model.initialState(), "", 1);
  const good = model.nextState(failed, feedWith([ITEM]), 0);

  assert.equal(good.status, "ok");
  assert.deepEqual(plain(good.items), [ITEM]);
});

test("bar text shows the glyph and the item count", () => {
  assert.equal(model.barText(model.initialState()), model.FEED_GLYPH);
  assert.equal(model.barText(model.parseFeed(feedWith([]), 0)), model.FEED_GLYPH);
  assert.equal(model.barText(model.parseFeed(fixture("valid.json"), 0)), `${model.FEED_GLYPH} 3`);
});

test("bar text in error shows the alert glyph, not a number", () => {
  const good = model.nextState(model.initialState(), fixture("valid.json"), 0);

  assert.equal(model.barText(model.nextState(good, "", 1)), model.ALERT_GLYPH);
  assert.equal(model.barText({ status: "incompatible", items: [] }), model.ALERT_GLYPH);
});

test("error messages are fixed texts, whatever the binary printed", () => {
  assert.equal(model.errorMessage("error"), model.ERROR_MESSAGE);
  assert.equal(model.errorMessage("incompatible"), model.INCOMPATIBLE_MESSAGE);
  for (const status of ["ok", "loading", "__proto__", "constructor", undefined]) {
    assert.equal(model.errorMessage(status), "", `status: ${status}`);
  }
});

test("hostile items are dropped and the prototype stays clean", () => {
  const state = model.parseFeed(fixture("hostile.json"), 0);

  assert.equal(state.status, "ok");
  assert.deepEqual(plain(state.items).map((item) => item.id), ["x:4"]);
  assert.equal(Object.getPrototypeOf(state.items[0]).polluted, undefined);
  assert.equal(vm.runInContext("({}).polluted", model), undefined);
  assert.equal({}.polluted, undefined);
});

test("items without a string id, title or url are dropped", () => {
  const items = [
    { ...ITEM, id: 1 },
    { ...ITEM, id: "" },
    { ...ITEM, title: null },
    { ...ITEM, url: undefined },
    { ...ITEM, source: 5 },
  ];
  const state = model.parseFeed(feedWith(items), 0);

  assert.deepEqual(plain(state.items), [{ ...ITEM, source: "" }]);
});

test("long titles and sources are cut without splitting a character", () => {
  const title = "a".repeat(298) + "😀😀😀";
  const [item] = model.parseFeed(feedWith([{ ...ITEM, title, source: "s".repeat(65) }]), 0).items;

  assert.equal(Array.from(item.title).length, 300);
  assert.ok(item.title.endsWith("😀…"));
  assert.equal(Array.from(item.source).length, 64);
});

test("a huge list is capped at 200 items", () => {
  const items = Array.from({ length: 10000 }, (_, index) => ({ ...ITEM, id: `x:${index}` }));
  const state = model.parseFeed(feedWith(items), 0);

  assert.equal(state.items.length, 200);
  assert.equal(state.items[199].id, "x:199");
});

test("only plain https links can be opened", () => {
  assert.equal(model.isOpenableUrl("https://example.com/a?b=c#d"), true);
  assert.equal(model.isOpenableUrl("https://example.com"), true);
  for (const url of [
    "file:///x",
    "http://x",
    " https://x",
    "https://x ",
    "javascript:alert(1)",
    "HTTPS://x",
    "https://",
    "https://bank.example@evil.example/",
    "https://x/\n",
    "https://x/\u0000",
    "https://x/" + "a".repeat(2048),
    null,
    {},
  ]) {
    assert.equal(model.isOpenableUrl(url), false, `url: ${JSON.stringify(url)}`);
  }
});
