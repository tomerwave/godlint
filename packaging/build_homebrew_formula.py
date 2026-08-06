from __future__ import annotations

import argparse
import os
import re
import tempfile
from pathlib import Path


REPOSITORY = "https://github.com/tomerwave/godlint"
TARGETS = ("aarch64-apple-darwin", "x86_64-apple-darwin")


def checksum_for(assets: Path, target: str) -> str:
    archive = f"godlint-{target}.tar.gz"
    checksum_file = assets / f"{archive}.sha256"

    try:
        contents = checksum_file.read_text(encoding="utf-8")
    except FileNotFoundError:
        raise SystemExit(f"missing checksum for {target}: {checksum_file}") from None

    match = re.fullmatch(rf"([0-9a-f]{{64}})  {re.escape(archive)}\n", contents)

    if match is None:
        raise SystemExit(f"{checksum_file} is not a SHA-256 checksum for {archive}")

    return match.group(1)


def formula(version: str, checksums: dict[str, str]) -> str:
    sections = []

    for target, condition in (
        ("aarch64-apple-darwin", "arm"),
        ("x86_64-apple-darwin", "intel"),
    ):
        archive = f"godlint-{target}.tar.gz"
        sections.append(
            f'''    if Hardware::CPU.{condition}?
      url "{REPOSITORY}/releases/download/v{version}/{archive}"
      sha256 "{checksums[target]}"

      define_method(:install) do
        bin.install "godlint"
      end
    end'''
        )

    sections_text = "\n\n".join(sections)

    return f'''class Godlint < Formula
  desc "Deterministic code-policy engine for polyglot repositories"
  homepage "{REPOSITORY}"
  version "{version}"
  license "MIT"

  on_macos do
{sections_text}
  end
end
'''


def write(path: Path, contents: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)

    with tempfile.NamedTemporaryFile(
        "w",
        encoding="utf-8",
        dir=path.parent,
        prefix=f".{path.name}.",
        delete=False,
    ) as temporary:
        temporary.write(contents)
        temporary_path = Path(temporary.name)

    os.replace(temporary_path, path)


def main() -> int:
    parser = argparse.ArgumentParser(description="Render the Homebrew formula for a Godlint release.")
    parser.add_argument("version")
    parser.add_argument("--assets", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    arguments = parser.parse_args()
    checksums = {target: checksum_for(arguments.assets, target) for target in TARGETS}
    write(arguments.out, formula(arguments.version, checksums))

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
