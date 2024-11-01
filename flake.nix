{
  description = "Nix dev environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    util.url = "github:QuartzLibrary/nix";
    util.flake = false; # Ignore lock file in QuartzLibrary/nix
    # util.url = "path:/home/user/code/nix"; # Optional: make Nix watch for changes in the local files
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      util,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (system: {
      devShells.default = import (util + "/shells/rust.nix") {
        pkgs = nixpkgs.legacyPackages.${system};
        rust-toolchain = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
      };
      formatter = nixpkgs.legacyPackages.${system}.nixfmt-rfc-style;
    });
}
