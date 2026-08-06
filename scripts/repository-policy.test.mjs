import assert from "node:assert/strict";
import test from "node:test";
import {
  validatePullRequest,
  validateReleaseWorkflow,
} from "./repository-policy.mjs";

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

test("rejects a release workflow that drops a binary platform", () => {
  const workflow = `
permissions:
  contents: read
  publish:
    if: github.event_name == 'push'
    permissions:
      contents: write
  x86_64-unknown-linux-gnu
  aarch64-unknown-linux-gnu
  x86_64-apple-darwin
  aarch64-apple-darwin
  x86_64-pc-windows-msvc
  git cat-file -t "$GITHUB_REF"
  git rev-parse "$GITHUB_REF^{commit}"
  scripts/release-policy.sh
  --notes-file "docs/releases/$RELEASE_REF.md"
  artifacts/SHA256SUMS
  actions/attest-build-provenance@v3
  --verify-tag
  gh release create
`;
  const errors = validateReleaseWorkflow(workflow);
  assert.ok(errors.some((error) => error.includes("Windows ARM64 binary")));
});
