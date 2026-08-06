import assert from "node:assert/strict";
import test from "node:test";
import { validatePullRequest } from "./repository-policy.mjs";

const ordinary = {
  title: "feat(cli): add export command",
  body: "Closes #42",
  head: "feat/42-export-command",
  base: "develop",
  actor: "contributor",
  headRepository: "contributor/banna",
  baseRepository: "owner/banna",
};

test("accepts an ordinary issue pull request to develop", () => {
  assert.deepEqual(validatePullRequest(ordinary), []);
});

test("rejects a mismatched issue and branch", () => {
  const errors = validatePullRequest({ ...ordinary, body: "Closes #43" });
  assert.ok(errors.some((error) => error.includes("source branch belongs to #42")));
});

test("accepts the canonical develop promotion", () => {
  assert.deepEqual(
    validatePullRequest({
      title: "release: promote banna v0.2.0 to main",
      body: "",
      head: "develop",
      base: "main",
      actor: "maintainer",
      headRepository: "owner/banna",
      baseRepository: "owner/banna",
    }),
    [],
  );
});

test("allows Dependabot only against develop", () => {
  assert.deepEqual(
    validatePullRequest({
      title: "build(deps): bump serde",
      body: "",
      head: "dependabot/cargo/serde-1.0.0",
      base: "develop",
      actor: "dependabot[bot]",
    }),
    [],
  );
});

