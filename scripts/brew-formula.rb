# SPDX-License-Identifier: Apache-2.0
class LukuidCli < Formula
  desc "Inspect and verify .luku forensic evidence packages"
  homepage "https://github.com/lukuid/cli"
  license "Apache-2.0"
  version "__VERSION__"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/lukuid/cli/releases/download/v__VERSION__/lukuid-cli-__VERSION__-aarch64-apple-darwin.tar.gz"
      sha256 "__MACOS_ARM64_SHA__"
    else
      url "https://github.com/lukuid/cli/releases/download/v__VERSION__/lukuid-cli-__VERSION__-x86_64-apple-darwin.tar.gz"
      sha256 "__MACOS_X64_SHA__"
    end
  end

  def install
    bin.install "lukuid-cli"
  end

  test do
    assert_match "lukuid-cli", shell_output("#{bin}/lukuid-cli --help")
  end
end
