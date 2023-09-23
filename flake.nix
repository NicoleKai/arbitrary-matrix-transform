{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk { };
        environment = (import ./environment.nix { inherit pkgs; });
      in
      rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage environment // {
          src = ./.;
        };

        devShell = pkgs.mkShell environment // {
        };
        # For `nix develop`:
        # devShell = pkgs.mkShell {
          # nativeBuildInputs = with pkgs; [ rustc cargo pkg-config alsaLib udev xorg.libX11 wayland ];
          # PKG_CONFIG_PATH = ''${pkgs.alsaLib}/lib/pkgconfig:${pkgs.udev}/lib/pkgconfig:\
          # ${pkgs.xorg.libX11}/lib/pkgconfig:${pkgs.wayland}/lib/pkgconfig'';
          # buildInputs = (import ./environment.nix { inherit pkgs; });
        # };
      }
    );
}
