<a href="https://github.com/nickfla1/nana"><img src="https://img.shields.io/github/workflow/status/nickfla1/nana/Rust"></a>
&nbsp;
<a href="https://crates.io/crates/nana"><img src="https://img.shields.io/crates/v/nana"></a>
&nbsp;
<a href="https://github.com/nickfla1/nana/blob/main/LICENSE"><img src="https://img.shields.io/github/license/nickfla1/nana"></a>

> **Warning**: Nana is still a Work In Progress and its usage in production environments is highly discouraged.

## Nana
Fast and customizable Node.js package manager written in Rust!

## Usage

#### `version`
Prints installed nana version.
```sh
nana version
# Outputs: `nana - v1.0.0`
```

#### `init`
Initializes a `package.json`.
```sh
# Default configuration
nana init

# Package name defaults to directory name, change it using `--name` or `-n`
name --name package-name
```

#### `install`
Installs the dependencies specified in the `package.json`
```sh
nana install
```

### Running cutsom scripts

TBD.

## Contributing

TBD.

## License

[LICENSE](LICENSE)
