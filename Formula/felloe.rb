class Felloe < Formula
    desc "A helm version manager"
    homepage "https://github.com/ddlees/felloe"
    url "https://github.com/ddlees/felloe/releases/download/v0.1.0/felloe-x86_64-darwin.tar.gz"
    sha256 "c2add43524b0a9a382d0f5e15339bdaaf51adaa7c73ae632596b3fc27aacb2ee"
    version "0.1.0"

    def install
      bin.install "felloe"
      bash_completion.install "completions/felloe.bash"
    end

    test do
      assert_match "-F _felloe",
      shell_output("source #{bash_completion}/felloe.bash && complete -p felloe")
    end
end
