#!/usr/bin/env python3
"""Build one Python wheel per platform from the binaries a release already produced.

A wheel is a zip with a known layout, so this repackages rather than rebuilds: the binary inside
each wheel is the identical file published to GitHub Releases and npm, already run and version
checked by the job that built it. Building again would ship something that was never tested.

pip picks a wheel by its platform tag, and unlike npm it distinguishes glibc from musl, so a
static musl binary cannot serve both -- each libc needs its own wheel. A binary placed in the
`.data/scripts/` directory is installed onto the path, which is what makes `godlint` a command.
"""

from __future__ import annotations

import argparse
import base64
import csv
import hashlib
import io
import zipfile
from pathlib import Path

# Rust target -> the platform tag pip matches against.
TAGS = {
    "aarch64-apple-darwin": "macosx_11_0_arm64",
    "x86_64-apple-darwin": "macosx_10_12_x86_64",
    "aarch64-unknown-linux-gnu": "manylinux_2_17_aarch64",
    "x86_64-unknown-linux-gnu": "manylinux_2_17_x86_64",
    "aarch64-unknown-linux-musl": "musllinux_1_2_aarch64",
    "x86_64-unknown-linux-musl": "musllinux_1_2_x86_64",
    "x86_64-pc-windows-msvc": "win_amd64",
}

NAME = "godlint"
SUMMARY = "A deterministic code-policy engine for polyglot repositories."
HOMEPAGE = "https://github.com/tomerwave/godlint"


def metadata(version: str, description: str) -> str:
    return (
        "Metadata-Version: 2.1\n"
        f"Name: {NAME}\n"
        f"Version: {version}\n"
        f"Summary: {SUMMARY}\n"
        f"Home-page: {HOMEPAGE}\n"
        "License: MIT\n"
        "Requires-Python: >=3.8\n"
        "Description-Content-Type: text/markdown\n"
        "Classifier: Development Status :: 3 - Alpha\n"
        "Classifier: License :: OSI Approved :: MIT License\n"
        "Classifier: Topic :: Software Development :: Quality Assurance\n"
        "\n"
        f"{description}"
    )


def wheel_metadata(tag: str) -> str:
    return (
        "Wheel-Version: 1.0\n"
        "Generator: godlint packaging/build_wheels.py\n"
        "Root-Is-Purelib: false\n"
        f"Tag: py3-none-{tag}\n"
    )


def digest(payload: bytes) -> str:
    encoded = base64.urlsafe_b64encode(hashlib.sha256(payload).digest()).rstrip(b"=")

    return f"sha256={encoded.decode('ascii')}"


def build(version: str, target: str, binary: Path, description: str, out: Path) -> Path:
    tag = TAGS[target]
    distribution = f"{NAME}-{version}"
    executable = "godlint.exe" if target.endswith("windows-msvc") else "godlint"
    entries = {
        f"{distribution}.data/scripts/{executable}": binary.read_bytes(),
        f"{distribution}.dist-info/METADATA": metadata(version, description).encode("utf-8"),
        f"{distribution}.dist-info/WHEEL": wheel_metadata(tag).encode("utf-8"),
    }

    record = io.StringIO()
    writer = csv.writer(record, lineterminator="\n")

    for path, payload in entries.items():
        writer.writerow([path, digest(payload), len(payload)])

    writer.writerow([f"{distribution}.dist-info/RECORD", "", ""])
    entries[f"{distribution}.dist-info/RECORD"] = record.getvalue().encode("utf-8")

    out.mkdir(parents=True, exist_ok=True)
    wheel = out / f"{distribution}-py3-none-{tag}.whl"

    with zipfile.ZipFile(wheel, "w", zipfile.ZIP_DEFLATED) as archive:
        for path, payload in entries.items():
            info = zipfile.ZipInfo(path)
            # pip decides a wheel entry is executable with stat.S_ISREG, so the mode needs the
            # regular-file bits and not permissions alone, and it needs a Unix create_system for
            # the mode to be read at all. Without both, pip installs a command it cannot execute.
            info.create_system = 3
            mode = 0o100755 if ".data/scripts/" in path else 0o100644
            info.external_attr = mode << 16
            archive.writestr(info, payload)

    return wheel


def binary_for(binaries: Path, target: str) -> Path:
    executable = "godlint.exe" if target.endswith("windows-msvc") else "godlint"
    candidate = binaries / f"binary-{target}" / executable

    if not candidate.is_file():
        raise SystemExit(f"no binary for {target} at {candidate}; a wheel without one installs no command")

    return candidate


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version")
    parser.add_argument("--binaries", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--only", action="append", help="build one wheel, for verifying locally")
    arguments = parser.parse_args()
    targets = arguments.only or list(TAGS)
    description = Path("README.md").read_text(encoding="utf-8")

    for target in targets:
        wheel = build(
            arguments.version,
            target,
            binary_for(arguments.binaries, target),
            description,
            arguments.out,
        )
        print(wheel.name)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
