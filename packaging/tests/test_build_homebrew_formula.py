from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GENERATOR = ROOT / "packaging" / "build_homebrew_formula.py"


class BuildHomebrewFormulaTest(unittest.TestCase):
    def test_renders_both_macos_architectures(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            assets = Path(directory) / "assets"
            output = Path(directory) / "Formula" / "godlint.rb"
            assets.mkdir()
            self.write_checksum(assets, "aarch64-apple-darwin", "a" * 64)
            self.write_checksum(assets, "x86_64-apple-darwin", "b" * 64)

            result = subprocess.run(
                [
                    "python3",
                    str(GENERATOR),
                    "0.7.0",
                    "--assets",
                    str(assets),
                    "--out",
                    str(output),
                ],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(
                output.read_text(encoding="utf-8"),
                self.expected_formula(),
            )

    def test_refuses_a_missing_architecture_checksum(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            assets = Path(directory) / "assets"
            output = Path(directory) / "Formula" / "godlint.rb"
            assets.mkdir()
            self.write_checksum(assets, "aarch64-apple-darwin", "a" * 64)

            result = self.run_generator(assets, output)

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("x86_64-apple-darwin", result.stderr)
            self.assertFalse(output.exists())

    def test_refuses_a_checksum_for_a_different_archive(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            assets = Path(directory) / "assets"
            output = Path(directory) / "Formula" / "godlint.rb"
            assets.mkdir()
            self.write_checksum(assets, "aarch64-apple-darwin", "a" * 64)
            (assets / "godlint-x86_64-apple-darwin.tar.gz.sha256").write_text(
                f"{'b' * 64}  godlint-aarch64-apple-darwin.tar.gz\n",
                encoding="utf-8",
            )

            result = self.run_generator(assets, output)

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("SHA-256", result.stderr)
            self.assertFalse(output.exists())

    def test_refuses_an_invalid_checksum(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            assets = Path(directory) / "assets"
            output = Path(directory) / "Formula" / "godlint.rb"
            assets.mkdir()
            self.write_checksum(assets, "aarch64-apple-darwin", "a" * 64)
            self.write_checksum(assets, "x86_64-apple-darwin", "not-a-checksum")

            result = self.run_generator(assets, output)

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("SHA-256", result.stderr)
            self.assertFalse(output.exists())

    def run_generator(self, assets: Path, output: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                "python3",
                str(GENERATOR),
                "0.7.0",
                "--assets",
                str(assets),
                "--out",
                str(output),
            ],
            check=False,
            capture_output=True,
            text=True,
        )

    def write_checksum(self, assets: Path, target: str, checksum: str) -> None:
        archive = f"godlint-{target}.tar.gz"
        (assets / f"{archive}.sha256").write_text(
            f"{checksum}  {archive}\n",
            encoding="utf-8",
        )

    def expected_formula(self) -> str:
        return """class Godlint < Formula
  desc "Deterministic code-policy engine for polyglot repositories"
  homepage "https://github.com/tomerwave/godlint"
  version "0.7.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tomerwave/godlint/releases/download/v0.7.0/godlint-aarch64-apple-darwin.tar.gz"
      sha256 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"

      define_method(:install) do
        bin.install "godlint"
      end
    end

    if Hardware::CPU.intel?
      url "https://github.com/tomerwave/godlint/releases/download/v0.7.0/godlint-x86_64-apple-darwin.tar.gz"
      sha256 "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"

      define_method(:install) do
        bin.install "godlint"
      end
    end
  end
end
"""


if __name__ == "__main__":
    unittest.main()
