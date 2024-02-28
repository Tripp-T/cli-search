{
	description = "A search cli";
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
		flake-utils.url = "github:numtide/flake-utils";
	};
	outputs = { self, nixpkgs, flake-utils }:
		flake-utils.lib.eachDefaultSystem
			(system: let
				pkgs = nixpkgs.legacyPackages.${system};
				shell = import ./shell.nix { inherit pkgs; };
			in {
				devShells.default = shell;
				# TODO: Configure builds and checks to be done via nix flake
			});
}
