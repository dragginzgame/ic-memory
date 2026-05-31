{
  description = "ic-memory whitepaper and Lean proof tooling";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        tex = pkgs.texlive.combined.scheme-small;
      in
      {
        packages.lean-proof-lab = pkgs.stdenvNoCC.mkDerivation {
          pname = "ic-memory-lean-proof-lab";
          version = "0.1.0";
          src = ./lean;
          nativeBuildInputs = [ pkgs.lean4 ];
          buildPhase = ''
            export HOME="$TMPDIR"
            lake build
          '';
          installPhase = ''
            mkdir -p "$out"
            cp -R . "$out/src"
          '';
        };

        packages.whitepaper-pdf = pkgs.stdenvNoCC.mkDerivation {
          pname = "ic-memory-whitepaper";
          version = "0.1.0";
          src = ./.;
          nativeBuildInputs = [
            pkgs.lean4
            tex
          ];
          buildPhase = ''
            export HOME="$TMPDIR"
            (cd lean && lake build)
            mkdir -p latex.out
            pdflatex -interaction=nonstopmode -halt-on-error -output-directory=latex.out ic-memory.tex
            pdflatex -interaction=nonstopmode -halt-on-error -output-directory=latex.out ic-memory.tex
          '';
          installPhase = ''
            mkdir -p "$out"
            cp latex.out/ic-memory.pdf "$out/"
          '';
        };

        packages.default = self.packages.${system}.whitepaper-pdf;

        checks.lean-proof-lab = self.packages.${system}.lean-proof-lab;
        checks.whitepaper-pdf = self.packages.${system}.whitepaper-pdf;

        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.lean4
            tex
          ];
          shellHook = ''
            echo "ic-memory whitepaper shell"
            echo "  lake build:      cd lean && lake build"
            echo "  pdf build:       nix build .#whitepaper-pdf"
            echo "  proof package:   nix build .#lean-proof-lab"
          '';
        };
      }
    );
}
