# authors

Print and format authors from ["name <email>"] lists

## Motivation
This utility is meant to be used in a pipeline with [toml-path](https://github.com/adam-gaia/toml-path),
for extracting only the names of authors without the emails
```ignore
$ toml-path --raw-output .package.authors Cargo.toml | authors --names
Adam Gaia
```

## Usage
- Without any flags, this utility is redundant
```console
$ authors "foo <foo@bar.com>, foo2 <foo2@bar.com>"
foo <foo@bar.com>, foo2 <foo2@bar.com>

```

- Print names only
```console
$ authors --names "foo <foo@bar.com>, foo2 <foo2@bar.com>"
foo, foo2

```

- Print emails only
```console
$ authors --emails "foo <foo@bar.com>, foo2 <foo2@bar.com>"
foo@bar.com, foo2@bar.com

```

- Piping to stdin instead of proving an input argument works too
```ignore
$ echo "foo <foo@bar.com>, foo2 <foo2@bar.com>" | authors --names
foo, foo2

```
