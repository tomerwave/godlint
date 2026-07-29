#!/usr/bin/env python3
"""Assemble the npm packages for a release from the binaries it built.

npm installs a platform package only when its `os` and `cpu` match, so a user downloads the one
binary they can run and nothing else. The front door is `@godlint/cli`: it carries no binary,
declares every platform package as optional, and runs whichever one npm chose. Nothing is fetched
during install, so this works with `--ignore-scripts` and without a network.

The front door is scoped because npm refuses the bare name `godlint` as too similar to `oxlint`.
The command is still `godlint`, since the executable a package installs is named independently of
the package.

Linux ships the statically linked musl build, so one binary per architecture runs against either
libc instead of failing on a loader error.
"""

from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

# Rust target -> the pair npm reports as process.platform and process.arch.
PLATFORMS = {
    "aarch64-apple-darwin": ("darwin", "arm64"),
    "x86_64-apple-darwin": ("darwin", "x64"),
    "aarch64-unknown-linux-musl": ("linux", "arm64"),
    "x86_64-unknown-linux-musl": ("linux", "x64"),
    "x86_64-pc-windows-msvc": ("win32", "x64"),
}

SCOPE = "@godlint"
REPOSITORY = "https://github.com/tomerwave/godlint"
DESCRIPTION = "A deterministic code-policy engine for polyglot repositories."
SHIM = Path(__file__).parent / "npm" / "shim.js"


def common(version: str) -> dict[str, object]:
    return {
        "version": version,
        "description": DESCRIPTION,
        "license": "MIT",
        "repository": {"type": "git", "url": f"{REPOSITORY}.git"},
        "homepage": f"{REPOSITORY}#readme",
        "keywords": ["lint", "linter", "static-analysis", "code-quality", "policy"],
    }


def write(path: Path, contents: dict[str, object]) -> None:
    path.write_text(json.dumps(contents, indent=2) + "\n", encoding="utf-8")


def platform_package(out: Path, version: str, target: str, binary: Path) -> str:
    system, architecture = PLATFORMS[target]
    name = f"{SCOPE}/cli-{system}-{architecture}"
    directory = out / f"cli-{system}-{architecture}"
    executable = "godlint.exe" if system == "win32" else "godlint"

    directory.mkdir(parents=True)
    shutil.copy2(binary, directory / executable)
    (directory / executable).chmod(0o755)
    (directory / "README.md").write_text(
        f"# {name}\n\nThe Godlint binary for {system} {architecture}. "
        f"Install [godlint]({REPOSITORY}#readme) instead of this package.\n",
        encoding="utf-8",
    )
    write(
        directory / "package.json",
        {
            "name": name,
            **common(version),
            "os": [system],
            "cpu": [architecture],
            "files": [executable, "README.md"],
        },
    )

    return name


def shim_package(out: Path, version: str, names: list[str]) -> None:
    directory = out / "cli"
    (directory / "bin").mkdir(parents=True)
    shutil.copy2(SHIM, directory / "bin" / "godlint.js")
    (directory / "bin" / "godlint.js").chmod(0o755)
    shutil.copy2("README.md", directory / "README.md")
    shutil.copy2("LICENSE", directory / "LICENSE")
    write(
        directory / "package.json",
        {
            "name": f"{SCOPE}/cli",
            **common(version),
            "bin": {"godlint": "bin/godlint.js"},
            "files": ["bin/godlint.js", "README.md", "LICENSE"],
            "optionalDependencies": dict.fromkeys(sorted(names), version),
        },
    )


def spelled(directory: Path) -> str:
    text = directory.as_posix()

    return text if text.startswith(("/", "./", "../")) else f"./{text}"


def binary_for(binaries: Path, target: str) -> Path:
    executable = "godlint.exe" if target.endswith("windows-msvc") else "godlint"
    candidates = (
        binaries / f"binary-{target}" / executable,
        binaries / target / executable,
        binaries / f"godlint-{target}" / executable,
    )

    for candidate in candidates:
        if candidate.is_file():
            return candidate

    raise SystemExit(
        f"no binary for {target} under {binaries}; npm would install a package that cannot run"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version")
    parser.add_argument("--binaries", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument(
        "--only",
        action="append",
        help="build one platform package, for verifying on a single machine",
    )
    arguments = parser.parse_args()
    targets = arguments.only or list(PLATFORMS)

    if arguments.out.exists():
        shutil.rmtree(arguments.out)

    names = [
        platform_package(
            arguments.out, arguments.version, target, binary_for(arguments.binaries, target)
        )
        for target in targets
    ]

    shim_package(arguments.out, arguments.version, names)

    order = [arguments.out / f"cli-{system}-{architecture}" for system, architecture in (
        PLATFORMS[target] for target in targets
    )] + [arguments.out / "cli"]
    (arguments.out / "publish-order").write_text(
        "".join(f"{spelled(directory)}\n" for directory in order), encoding="utf-8"
    )

    print(f"{len(order)} packages in {arguments.out}, publish order in publish-order")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
