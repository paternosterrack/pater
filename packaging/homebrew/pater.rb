class Pater < Formula
  desc "CLI for Paternoster Rack marketplace"
  homepage "https://github.com/paternosterrack/pater"
  url "https://github.com/paternosterrack/pater/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_RELEASE_SHA256"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match "pater", shell_output("#{bin}/pater --version")
  end
end
