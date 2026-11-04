// T19–T21 and S13 of docs/test-strategy/002-kev-poc-cache.md.
import assert from "node:assert/strict";
import { test } from "node:test";

import { loadModel } from "./load-model.mjs";

const model = loadModel();

// Objects made inside the vm have another realm's prototypes; compare data only.
function plain(value) {
  return JSON.parse(JSON.stringify(value));
}

const KEV_ITEM = {
  id: "kev:CVE-2026-1234",
  category: "vulnerability",
  title: "CVE-2026-1234 Example Vulnerability",
  url: "https://nvd.nist.gov/vuln/detail/CVE-2026-1234",
  source: "CISA KEV",
  publishedAt: "2026-11-03",
  pocCount: 3,
  knownRansomwareUse: true,
};

function parse(document) {
  return model.parseFeed(JSON.stringify({ schemaVersion: 1, items: [], ...document }), 0);
}

function parseItem(fields) {
  return plain(parse({ items: [{ ...KEV_ITEM, ...fields }] }).items[0]);
}

test("valid optional fields are kept", () => {
  assert.deepEqual(parseItem({}), KEV_ITEM);
  assert.equal(parseItem({ pocCount: 0, knownRansomwareUse: false }).pocCount, 0);
  assert.equal(parseItem({ knownRansomwareUse: false }).knownRansomwareUse, false);
});

test("items without the optional fields stay as they were", () => {
  const { publishedAt, pocCount, knownRansomwareUse, ...basic } = KEV_ITEM;
  assert.deepEqual(plain(parse({ items: [basic] }).items[0]), basic);
});

test("hostile optional fields are dropped, the item stays", () => {
  for (const pocCount of ["3", 1e309, -1, 10001, 2.5, null, true]) {
    assert.equal(parseItem({ pocCount }).pocCount, undefined, `pocCount: ${pocCount}`);
  }
  for (const publishedAt of ["<b>", "2026-13-01", "2026-11-3", " 2026-11-03", 20261103]) {
    assert.equal(parseItem({ publishedAt }).publishedAt, undefined, `publishedAt: ${publishedAt}`);
  }
  for (const knownRansomwareUse of ["true", 1, null]) {
    assert.equal(parseItem({ knownRansomwareUse }).knownRansomwareUse, undefined);
  }
  assert.equal(parseItem({ pocCount: "3" }).id, KEV_ITEM.id);
});

test("detail line joins the parts the item has", () => {
  assert.equal(model.itemDetail(KEV_ITEM), "CISA KEV · 2026-11-03 · 3 PoC · ransomware");
  assert.equal(model.itemDetail({ source: "CISA KEV" }), "CISA KEV");
  assert.equal(
    model.itemDetail({ ...KEV_ITEM, pocCount: 0, knownRansomwareUse: false }),
    "CISA KEV · 2026-11-03 · 0 PoC",
  );
  assert.equal(model.itemDetail({ source: "", publishedAt: "2026-11-03" }), "2026-11-03");
  assert.equal(model.itemDetail(null), "");
});

test("failed sources become a notice and leave the bar icon alone", () => {
  const state = parse({ items: [KEV_ITEM], failedSources: ["CISA KEV"] });

  assert.equal(state.status, "ok");
  assert.equal(model.failureNotice(state), "Could not update: CISA KEV");
  assert.equal(model.barText(state), model.FEED_GLYPH + " 1");
});

test("no failed sources means no notice", () => {
  assert.equal(model.failureNotice(parse({})), "");
  assert.equal(model.failureNotice(model.initialState()), "");
  assert.equal(model.failureNotice(model.nextState(model.initialState(), "", 1)), "");
  assert.equal(model.failureNotice(null), "");
});

test("hostile failed sources are capped and cleaned", () => {
  const many = Array.from({ length: 1000 }, (_, index) => `Source ${index}`);
  assert.equal(plain(parse({ failedSources: many }).failedSources).length, 10);

  const mixed = parse({ failedSources: ["", 5, null, "x".repeat(100), "CISA KEV"] });
  const names = plain(mixed.failedSources);
  assert.equal(names.length, 2);
  assert.equal(Array.from(names[0]).length, 64);
  assert.equal(names[1], "CISA KEV");

  for (const failedSources of ["CISA KEV", { 0: "x" }, 5]) {
    assert.deepEqual(plain(parse({ failedSources }).failedSources), []);
  }
});
