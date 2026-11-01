import assert from "node:assert/strict";
import { test } from "node:test";

import { loadModel } from "./load-model.mjs";

const model = loadModel();

test("bar label is empty when there is nothing new", () => {
  assert.equal(model.barLabel(0), "");
});

test("bar label shows the count", () => {
  assert.equal(model.barLabel(3), "3");
});

test("bar label caps large counts at 99+", () => {
  assert.equal(model.barLabel(99), "99");
  assert.equal(model.barLabel(150), "99+");
});

test("bar label is empty for negative, fractional or non-number input", () => {
  for (const input of [-1, 2.5, NaN, Infinity, "3", null, undefined, {}]) {
    assert.equal(model.barLabel(input), "", `input: ${String(input)}`);
  }
});
