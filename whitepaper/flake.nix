{
  description = "ic-memory mdBook whitepaper and Lean proof tooling";

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

        packages.whitepaper-html = pkgs.stdenvNoCC.mkDerivation {
          pname = "ic-memory-whitepaper";
          version = "0.1.0";
          src = ./.;
          nativeBuildInputs = [
            pkgs.lean4
            pkgs.mdbook
            pkgs.mdbook-katex
          ];
          buildPhase = ''
            export HOME="$TMPDIR"
            (cd lean && lake build)
            mdbook build
          '';
          installPhase = ''
            mkdir -p "$out"
            cp -R book/* "$out/"
          '';
        };

        packages.default = self.packages.${system}.whitepaper-html;

        checks.lean-proof-lab = self.packages.${system}.lean-proof-lab;
        checks.whitepaper-html = self.packages.${system}.whitepaper-html;

        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.lean4
            pkgs.mdbook
            pkgs.mdbook-katex
          ];
          shellHook = ''
            echo "ic-memory whitepaper shell"
            echo "  lake build:      cd lean && lake build"
            echo "  mdBook build:    mdbook build"
            echo "  proof package:   nix build .#lean-proof-lab"
            echo "  HTML package:    nix build .#whitepaper-html"
          '';
        };
      }
    );
}
