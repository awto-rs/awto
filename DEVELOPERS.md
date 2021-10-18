# Developing awto

### Forking awto on Github

To contribute code to awto, you must have a GitHub account so you can push code to your own
fork of awto and open Pull Requests in the [GitHub Repository][github].

To create a Github account, follow the instructions [here](https://github.com/signup/free).
Afterwards, go ahead and [fork](http://help.github.com/forking) the
[main AngularJS repository][github].

### Building awto

To build awto, you clone the source code repository and use Cargo to build the libraries:

```shell
# Clone your Github repository:
git clone https://github.com/<github username>/awto.git

# Go to the awto directory:
cd awto

# Add the main awto repository as an upstream remote to your repository:
git remote add upstream "https://github.com/awto-rs/awto.git"

# Build awto packages:
cargo build -p <package> # package can be awto, awto-cli, awto-compile or awto-macros
```

## <a name="tests"> Running Tests

Tests can be run through Cargo with:

```shell
cargo run test -p <package>
```

To run all tests, omit the -p flag:

```shell
cargo test
```

## <a name="rules"></a> Coding Rules

To ensure consistency throughout the source code, keep these rules in mind as you are working:

- All features or bug fixes **must be tested** through doc tests or unit tests.
- All public API methods **must be documented** with code comments.

## <a name="commits"></a> Git Commit Guidelines

We have very precise rules over how our git commit messages can be formatted. This leads to **more
readable messages** that are easy to follow when looking through the **project history**. But also,
we use the git commit messages to **generate the awto change log**.

### Commit Message Format

Each commit message consists of a **header**, a **body** and a **footer**. The header has a special
format that includes a **type**, a **scope** and a **subject**:

```
<type>(<scope>): <subject>
<BLANK LINE>
<body>
<BLANK LINE>
<footer>
```

The **header** is mandatory and the **scope** of the header is optional.

Any line of the commit message cannot be longer than 100 characters! This allows the message to be easier
to read on GitHub as well as in various git tools.

### Revert

If the commit reverts a previous commit, it should begin with `revert: `, followed by the header
of the reverted commit.
In the body it should say: `This reverts commit <hash>.`, where the hash is the SHA of the commit
being reverted.

### Type

Must be one of the following:

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, missing
  semi-colons, etc)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing or correcting existing tests
- **chore**: Changes to the build process or auxiliary tools and libraries such as documentation
  generation

### Scope

The scope could be anything specifying place of the commit change. For example `awto`,
`awto-cli`, `awto-compile`, `awto-macros`, etc...

You can omit the scope when the change affects more than a single scope.

### Subject

The subject contains succinct description of the change:

- use the imperative, present tense: "change" not "changed" nor "changes"
- don't capitalize first letter
- no dot (.) at the end

### Body

Just as in the **subject**, use the imperative, present tense: "change" not "changed" nor "changes".
The body should include the motivation for the change and contrast this with previous behavior.

### Footer

The footer should contain any information about **Breaking Changes** and is also the place to
[reference GitHub issues that this commit closes][closing-issues].

**Breaking Changes** should start with the word `BREAKING CHANGE:` with a space or two newlines.
The rest of the commit message is then used for this.

A detailed explanation can be found in this [document][commit-message-format].

## <a name="documentation"></a> Writing Documentation

Documentation should be written as doc comments and follow the guides explained on the [rustdoc book](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html).

### Building and viewing the docs locally

The docs can be built using Cargo:

```shell
cargo doc -p <package> --open
```
