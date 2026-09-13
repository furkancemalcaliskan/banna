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

const funding = {
  title: "chore(repository): add funding configuration",
  body: "Adds the repository funding configuration.",
  head: "chore/funding-configuration-develop",
  base: "develop",
  actor: "maintainer",
  headRepository: "owner/banna",
  baseRepository: "owner/banna",
};

test("accepts an ordinary issue pull request to develop", () => {
  assert.deepEqual(validatePullRequest(ordinary), []);
});

test("rejects a mismatched issue and branch", () => {
  const errors = validatePullRequest({ ...ordinary, body: "Closes #43" });
  assert.ok(errors.some((error) => error.includes("source branch belongs to #42")));
});

test("accepts the designated issue-free funding maintenance branches", () => {
  assert.deepEqual(validatePullRequest(funding), []);
  assert.deepEqual(
    validatePullRequest({
      ...funding,
      head: "chore/funding-configuration-main",
      base: "main",
    }),
    [],
  );
});

test("rejects funding maintenance with the wrong target or title", () => {
  assert.ok(
    validatePullRequest({ ...funding, base: "main" }).some((error) =>
      error.includes("must target develop"),
    ),
  );
  assert.ok(
    validatePullRequest({ ...funding, title: "chore(repository): add sponsors" }).some((error) =>
      error.includes("funding maintenance title must be"),
    ),
  );
});

test("rejects funding maintenance from a fork or with an issue closure", () => {
  assert.ok(
    validatePullRequest({ ...funding, headRepository: "contributor/banna" }).some((error) =>
      error.includes("must originate from this repository"),
    ),
  );
  const closingBodies = [
    "Closes #42",
    "Fixes owner/banna#42",
    "Resolves https://github.com/owner/banna/issues/42",
  ];
  for (const body of closingBodies) {
    assert.ok(
      validatePullRequest({ ...funding, body }).some((error) =>
        error.includes("must not close an issue"),
      ),
    );
  }
});

test("does not generalize the funding exception to lookalike branches", () => {
  const errors = validatePullRequest({
    ...funding,
    head: "chore/funding-configuration-other",
  });
  assert.ok(errors.some((error) => error.includes("ordinary issue branch must match")));
  assert.ok(errors.some((error) => error.includes("exactly one: Closes #<issue>")));
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
