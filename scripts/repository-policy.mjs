import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";

const CHANGE_TYPES = [
  "feat",
  "fix",
  "refactor",
  "test",
  "docs",
  "ci",
  "release",
  "chore",
];
const TYPE_PATTERN = CHANGE_TYPES.join("|");
const PULL_REQUEST_TITLE = new RegExp(
  `^(?:${TYPE_PATTERN})\\([a-z0-9][a-z0-9._/-]*\\): \\S(?:.*\\S)?$`,
);
const ISSUE_BRANCH = new RegExp(
  `^(?:${TYPE_PATTERN})/([1-9][0-9]*)-[a-z0-9]+(?:[._-][a-z0-9]+)*$`,
);
const RELEASE_TITLE =
  /^release: promote banna v[0-9]+\.[0-9]+\.[0-9]+ to main$/;

const REQUIRED_FILES = [
  "AGENTS.md",
  "CHANGELOG.md",
  "CODE_OF_CONDUCT.md",
  "CONTRIBUTING.md",
  "LICENSE",
  "README.md",
  "SECURITY.md",
  "SUPPORT.md",
  "docs/architecture.md",
  "docs/getting-started.md",
  "docs/maintainers.md",
  "docs/releasing.md",
];

const RELEASE_WORKFLOW_CONTRACTS = [
  ["Linux x86-64 binary", "x86_64-unknown-linux-gnu"],
  ["Linux ARM64 binary", "aarch64-unknown-linux-gnu"],
  ["macOS Intel binary", "x86_64-apple-darwin"],
  ["macOS Apple Silicon binary", "aarch64-apple-darwin"],
  ["Windows x86-64 binary", "x86_64-pc-windows-msvc"],
  ["Windows ARM64 binary", "aarch64-pc-windows-msvc"],
  ["annotated tag verification", 'git cat-file -t "$GITHUB_REF"'],
  ["tag commit identity verification", 'git rev-parse "$GITHUB_REF^{commit}"'],
  ["release policy validation", "scripts/release-policy.sh"],
  ["versioned release notes", '--notes-file "docs/releases/$RELEASE_REF.md"'],
  ["aggregate checksums", "artifacts/SHA256SUMS"],
  ["binary provenance attestation", "actions/attest-build-provenance@v3"],
  ["existing tag verification", "--verify-tag"],
  ["GitHub Release publication", "gh release create"],
];

function readText(file) {
  return fs.readFileSync(file, "utf8");
}

function stripFencedCode(markdown) {
  const output = [];
  let fence = null;
  for (const line of markdown.split("\n")) {
    const match = line.match(/^\s{0,3}(`{3,}|~{3,})/);
    if (match) {
      const marker = match[1][0];
      fence = fence === null ? marker : fence === marker ? null : fence;
    } else if (fence === null) {
      output.push(line);
    }
  }
  return output.join("\n");
}

function markdownDestinations(markdown) {
  const content = stripFencedCode(markdown);
  const destinations = [];
  for (const match of content.matchAll(/!?\[[^\]]*]\(([^)]+)\)/g)) {
    destinations.push(match[1]);
  }
  for (const match of content.matchAll(/^\s*\[[^\]]+]:\s*(\S+)/gm)) {
    destinations.push(match[1]);
  }
  return destinations;
}

function normalizedLocalDestination(rawDestination) {
  let destination = rawDestination.trim();
  if (destination.startsWith("<")) {
    const closing = destination.indexOf(">");
    destination = closing === -1 ? destination : destination.slice(1, closing);
  } else {
    destination = destination.split(/\s+/, 1)[0];
  }
  if (
    destination === "" ||
    destination.startsWith("#") ||
    /^[A-Za-z][A-Za-z0-9+.-]*:/.test(destination)
  ) {
    return null;
  }
  const withoutFragment = destination.split("#", 1)[0].split("?", 1)[0];
  if (withoutFragment === "") return null;
  try {
    return decodeURIComponent(withoutFragment);
  } catch {
    return withoutFragment;
  }
}

function markdownFiles(root) {
  const files = [];
  const visit = (directory) => {
    if (!fs.existsSync(directory)) return;
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) visit(file);
      else if (entry.isFile() && entry.name.endsWith(".md")) files.push(file);
    }
  };
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    if (entry.isFile() && entry.name.endsWith(".md")) {
      files.push(path.join(root, entry.name));
    }
  }
  visit(path.join(root, "docs"));
  const pullRequestTemplate = path.join(root, ".github", "PULL_REQUEST_TEMPLATE.md");
  if (fs.existsSync(pullRequestTemplate)) files.push(pullRequestTemplate);
  return [...new Set(files)].sort();
}

export function validateRepository(root) {
  const errors = [];
  for (const required of REQUIRED_FILES) {
    const file = path.join(root, required);
    if (!fs.existsSync(file) || !fs.statSync(file).isFile()) {
      errors.push(`required repository file is missing: ${required}`);
    }
  }
  const claudeFile = path.join(root, "CLAUDE.md");
  if (fs.existsSync(claudeFile)) {
    const first = readText(claudeFile).split("\n").find((line) => line.trim() !== "");
    if (first?.trim() !== "@AGENTS.md") {
      errors.push("CLAUDE.md must import @AGENTS.md as its first instruction");
    }
  }
  for (const file of markdownFiles(root)) {
    const relativeFile = path.relative(root, file);
    for (const rawDestination of markdownDestinations(readText(file))) {
      const destination = normalizedLocalDestination(rawDestination);
      if (destination === null) continue;
      const resolved = destination.startsWith("/")
        ? path.join(root, destination.slice(1))
        : path.resolve(path.dirname(file), destination);
      const relativeResolved = path.relative(root, resolved);
      if (
        relativeResolved === ".." ||
        relativeResolved.startsWith(`..${path.sep}`) ||
        path.isAbsolute(relativeResolved)
      ) {
        errors.push(`${relativeFile} references a path outside the repository: ${destination}`);
      } else if (!fs.existsSync(resolved)) {
        errors.push(`${relativeFile} references missing relative path: ${destination}`);
      }
    }
  }
  const releaseWorkflow = path.join(root, ".github", "workflows", "release.yml");
  if (!fs.existsSync(releaseWorkflow)) {
    errors.push("release workflow is missing: .github/workflows/release.yml");
  } else {
    errors.push(...validateReleaseWorkflow(readText(releaseWorkflow)));
  }
  return errors;
}

export function validateReleaseWorkflow(workflow) {
  const errors = [];
  for (const [description, contract] of RELEASE_WORKFLOW_CONTRACTS) {
    if (!workflow.includes(contract)) {
      errors.push(`release workflow is missing ${description}: ${contract}`);
    }
  }
  if (!/publish:\s*\n[\s\S]*?if: github\.event_name == 'push'/m.test(workflow)) {
    errors.push("release publication must be restricted to tag push events");
  }
  if (!/^permissions:\s*\n  contents: read\s*$/m.test(workflow)) {
    errors.push("release workflow must default to contents: read");
  }
  const writePermissions = workflow.match(/contents: write/g) ?? [];
  if (writePermissions.length !== 1) {
    errors.push("only the release publication job may receive contents: write");
  }
  if (/cargo publish|CARGO_REGISTRY_TOKEN|docker push/.test(workflow)) {
    errors.push("release workflow must not publish crates or containers");
  }
  return errors;
}

function closingIssues(body) {
  return [...body.matchAll(/\bCloses\s+#([1-9][0-9]*)\b/gi)].map((match) => match[1]);
}

export function validatePullRequest(metadata) {
  const title = metadata.title ?? "";
  const body = metadata.body ?? "";
  const head = metadata.head ?? "";
  const base = metadata.base ?? "";
  const actor = metadata.actor ?? "";
  const headRepository = metadata.headRepository ?? "";
  const baseRepository = metadata.baseRepository ?? "";
  const errors = [];

  if (actor === "dependabot[bot]" && head.startsWith("dependabot/")) {
    if (base !== "develop") errors.push(`Dependabot pull requests must target develop; received: ${base}`);
    return errors;
  }
  if (base === "main") {
    if (head !== "develop") errors.push(`only develop may be promoted to main; received: ${head}`);
    if (headRepository && baseRepository && headRepository !== baseRepository) {
      errors.push("release promotion must originate from this repository");
    }
    if (!RELEASE_TITLE.test(title)) {
      errors.push("release promotion title must match: release: promote banna vX.Y.Z to main");
    }
    return errors;
  }
  if (base !== "develop") {
    errors.push(`ordinary pull requests must target develop; received: ${base}`);
    return errors;
  }
  if (!PULL_REQUEST_TITLE.test(title)) {
    errors.push("ordinary pull request title must match: <type>(<scope>): <description>");
  }
  const branch = head.match(ISSUE_BRANCH);
  if (branch === null) {
    errors.push("ordinary issue branch must match: <type>/<issue>-<short-description>");
  }
  const issues = closingIssues(body);
  if (issues.length !== 1) {
    errors.push("ordinary pull request body must contain exactly one: Closes #<issue>");
  } else if (branch !== null && issues[0] !== branch[1]) {
    errors.push(`pull request closes #${issues[0]} but source branch belongs to #${branch[1]}`);
  }
  return errors;
}

export function pullRequestMetadata(event) {
  if (event.pull_request === undefined) throw new Error("event does not contain pull_request metadata");
  return {
    title: event.pull_request.title,
    body: event.pull_request.body,
    head: event.pull_request.head?.ref,
    base: event.pull_request.base?.ref,
    actor: event.pull_request.user?.login ?? event.sender?.login,
    headRepository: event.pull_request.head?.repo?.full_name,
    baseRepository: event.pull_request.base?.repo?.full_name,
  };
}

function optionValue(args, option) {
  const index = args.indexOf(option);
  return index === -1 || index + 1 >= args.length ? null : args[index + 1];
}

function report(name, errors) {
  if (errors.length === 0) {
    process.stdout.write(`${name}: ok\n`);
    return;
  }
  for (const error of errors) process.stderr.write(`${name} violation: ${error}\n`);
  process.exitCode = 1;
}

function run(args) {
  const root = path.resolve(optionValue(args, "--root") ?? ".");
  report("repository policy", validateRepository(root));
  const eventPath = optionValue(args, "--event");
  if (eventPath !== null) {
    const event = JSON.parse(readText(path.resolve(eventPath)));
    report("pull request policy", validatePullRequest(pullRequestMetadata(event)));
  }
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    run(process.argv.slice(2));
  } catch (error) {
    process.stderr.write(`repository policy error: ${error.message}\n`);
    process.exitCode = 1;
  }
}
