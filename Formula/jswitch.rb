class Jswitch < Formula
  desc "Fast Java version switcher written in Rust"
  homepage "https://github.com/m1ggle/jswitch"
  version "0.1.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/m1ggle/jswitch/releases/download/v0.1.0/jswitch-aarch64-apple-darwin.tar.gz"
      sha256 "PUT_AARCH64_SHA256_HERE"
    else
      url "https://github.com/m1ggle/jswitch/releases/download/v0.1.0/jswitch-x86_64-apple-darwin.tar.gz"
      sha256 "PUT_X86_64_SHA256_HERE"
    end
  end

  def install
    bin.install "jswitch"
    generate_completions_from_executable(bin/"jswitch", "completion")
  end

  test do
    assert_match "jswitch", shell_output("#{bin}/jswitch --help")
  end
end
