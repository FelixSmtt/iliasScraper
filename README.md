# iliasScraper

A Rust library for scraping data from the ILIAS learning management system. It provides functionality for session management, configuration of courses, and downloading course materials.

## Features

- Session management with support for Shibboleth authentication (Currently only for KIT)
- JSON-based configuration for courses via CLI
- Downloading course materials with progress indication
- Syncing course materials to a local directory

## Installation

````

### Build from source

1. Clone the repository:

```bash
git clone
cd iliasScraper
````

2. Build the project using Cargo:

```bash
cargo run -r -- [COMMAND]
```

## Usage

```
Usage: ilias [COMMAND]

Commands:
  tree    Fetch a course tree
  sync    Sync courses to local storage
  cli     Start interactive command line interface
  config  Show or edit configuration
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Configuration

The configuration is stored in a JSON file located at `~/.config/ilias/config.json`. You can edit this file directly or use the CLI to manage your courses and settings.

```json
{
  "path": "<path_to_download_directory>",
  "courses": [
    {
      "name": "<course_name>", // Directory and display name for the course
      "id": 1234567 // ILIAS course ID
    }
  ]
}
```

## Flake for NixOS

There is a flake available for NixOS users. You can add it to your `flake.nix` like this:

```nix
{
  description = "A Rust library for scraping data from the ILIAS learning management system";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/release-25.11";
    iliasScraper = {
        url = "github:FelixSmtt/iliasScraper";
        inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, iliasScraper, ... }: {
    nixosConfigurations.your-hostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./configuration.nix
        {
            # Inside your configuration.nix, you can reference the package like this:
            environment.systemPackages = [
                # Reference the 'default' package from your ilias flake
                inputs.iliasScraper.packages.${pkgs.system}.default
            ];
        }
      ];
    };
  };
}
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
