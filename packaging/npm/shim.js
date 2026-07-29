#!/usr/bin/env node
"use strict";

// npm installed exactly one platform package, the one matching os and cpu, so this finds that
// package rather than guessing a path. Nothing is downloaded here: the binary is already on disk,
// which is what lets the install work with --ignore-scripts and without a network.
const { spawnSync } = require("node:child_process");
const path = require("node:path");

const suffix = process.platform === "win32" ? ".exe" : "";
const name = `@godlint/cli-${process.platform}-${process.arch}`;

let binary;

try {
  binary = path.join(path.dirname(require.resolve(`${name}/package.json`)), `godlint${suffix}`);
} catch {
  console.error(
    `godlint: no prebuilt binary for ${process.platform}-${process.arch}.`,
    `\nInstall ${name} directly, or build from source with \`cargo install godlint-cli\`.`
  );
  process.exit(1);
}

const finished = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });

if (finished.error) {
  console.error(`godlint: could not run ${binary}: ${finished.error.message}`);
  process.exit(1);
}

// A signalled child reports a null status, which must not become a success.
process.exit(finished.status === null ? 1 : finished.status);
