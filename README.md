# Forge

Forge is a NestJS artifact generator written in Rust. It helps you scaffold TypeScript files for NestJS projects with consistent naming, sensible defaults, optional spec generation, and project-aware output paths.

It is built as a Rust workspace with separate crates for the CLI, configuration loading, template rendering, core generation logic, and integration tests.

## Highlights

- Generates common NestJS artifacts such as modules, services, controllers, DTOs, guards, pipes, interceptors, middleware, decorators, strategies, interfaces, filters, config files, resolvers, entities, and classes.
- Supports NestJS-friendly naming transforms automatically from a single input name.
- Uses `forge.json` to discover the project root and resolve output paths.
- Can generate spec files alongside artifacts by default.
- Supports `--dry-run` previews, `--flat` output, and per-artifact path overrides.
- Includes shell completion generation for Bash, Zsh, Fish, and PowerShell.

## Requirements

- Rust 1.90 or newer
- A NestJS project or another TypeScript project that follows the same file layout conventions

## Installation

### Build from source

```bash
git clone https://github.com/Boluwatife-AJB/nest-forge.git
cd nest-forge
cargo build --release
```

The binary is built as `forge` in `target/release/forge`.

### Run without installing

```bash
cargo run -- generate service users
```

## Quick Start

Initialize configuration in the current project:

```bash
forge init
```

Generate a service:

```bash
forge generate service users
```

Use the short alias for the same command:

```bash
forge g service users
```

Preview what would be created without writing files:

```bash
forge g controller auth --dry-run
```

Generate files directly in the configured output path:

```bash
forge g dto create-user --flat
```

## Commands

### `forge generate` / `forge g`

Generate a NestJS artifact.

```bash
forge generate <artifact> <name> [options]
```

Common options:

- `--dry-run`: show output without writing files
- `--flat`: write files directly into the output path instead of a nested folder
- `--path <PATH>`: override the output path for this run
- `--spec=<BOOL>`: control spec generation for this run

Example:

```bash
forge generate module auth
forge generate controller products --spec=false
forge generate service product-catalog --path src/app
```

### `forge init`

Create a starter `forge.json` in the current directory.

```bash
forge init
```

If a config file already exists, Forge exits with an error instead of overwriting it.

### `forge info`

Show the resolved project root, configuration, supported artifact types, and completion hint.

```bash
forge info
```

### `forge completions`

Generate shell completions.

```bash
forge completions bash
forge completions zsh
forge completions fish
forge completions powershell
```

## Supported Artifacts

Forge currently supports these artifact kinds:

- `module` (`mo`)
- `service` (`s`)
- `controller` (`co`)
- `class` (`cl`)
- `dto`
- `guard` (`gu`)
- `interceptor` (`itc`)
- `middleware` (`mi`)
- `pipe` (`pi`)
- `decorator` (`d`)
- `strategy`
- `interface` (`itf`)
- `filter` (`f`)
- `config`
- `resolver` (`r`)
- `entity` (`e`)

## Configuration

Forge discovers a `forge.json` file in the project root and merges it with CLI options.

Example configuration:

```json
{
  "sourceRoot": "src",
  "language": "ts",
  "generateSpec": true,
  "flat": false,
  "paths": {
    "entity": "src/database/entities",
    "controller": "src/http/controllers"
  }
}
```

### Configuration fields

- `sourceRoot`: base folder used when no artifact-specific path override is set
- `language`: language label stored in config metadata
- `generateSpec`: enables or disables spec file generation by default
- `flat`: controls whether generated files are nested inside a folder named after the artifact
- `paths`: per-artifact path overrides keyed by artifact name

## How Output Is Structured

By default, Forge writes generated files into a folder named after the artifact name under the resolved output path.

Example:

```bash
forge g service products
```

Produces files like:

```text
src/products/products.service.ts
src/products/products.service.spec.ts
```

With `--flat`, the generated files are written directly into the resolved output path:

```text
src/products.service.ts
src/products.service.spec.ts
```

## Naming Conventions

Forge derives PascalCase, camelCase, snake_case, and kebab-case variants from a single input name.

For example, `product-category` becomes `ProductCategory`, `productCategory`, `product_category`, and `product-category`.

That lets templates use the same input for class names, file names, routes, and identifiers without manual editing.

## Examples

Generate a NestJS module:

```bash
forge g module auth
```

Generate a controller without a spec file:

```bash
forge g controller users --spec=false
```

Generate an entity into a custom location configured by `forge.json`:

```bash
forge g entity product
```

Preview a DTO scaffold:

```bash
forge g dto create-user --dry-run
```

## Development

The workspace is split into multiple crates:

- `forge-cli`: command-line interface and output formatting
- `forge-config`: discovery, parsing, and merging of `forge.json`
- `forge-core`: artifact resolution and filesystem generation
- `forge-template`: embedded Tera templates
- `forge-tests`: end-to-end CLI integration tests

Useful commands:

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt
```

### Quality Gates

The repository CI runs formatting, Clippy, unit and integration tests, snapshot tests, property tests, MSRV checks, dependency audits, and a release build verification.

## Release Flow

Releases are driven by tags and the workspace release configuration:

- `release.toml` consolidates commits and pushes, and tags releases as `v<version>`
- CI builds release artifacts and the release workflow publishes GitHub Releases from tagged builds

## Shell Completions

See [docs/completions.md](docs/completions.md) for install examples on Bash, Zsh, Fish, and PowerShell.

## License

Licensed under the MIT License.
