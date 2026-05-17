Contributing to Feder
=====================

Thank you for your interest in contributing to feder. This document outlines
the development workflow and the tools we use to maintain code quality.


Prerequisites
-------------

We use mise to manage our development environment and tasks. Before you start,
ensure you have the following installed:

 -  mise
 -  Rust (managed via mise)

Once mise is installed, you can set up the project by running:

~~~~ bash
mise install
~~~~


Development workflow
--------------------

We use mise tasks to automate common development steps. Please ensure your
changes pass the automated checks before submitting a pull request.

### Code formatting

We use `rustfmt` to maintain a consistent coding style. You can automatically
format your code by running:

~~~~ bash
mise run fmt
~~~~

### Quality checks

Before pushing your changes, run the full suite of quality checks. This
includes type checking, linting with Clippy, and verifying that the code is
correctly formatted.

~~~~ bash
mise run check
~~~~

We configure Clippy to treat all warnings as errors in `Cargo.toml`. This
ensures that the codebase remains clean and free of common pitfalls. If
`mise run check` fails, please address the reported issues before proceeding.

### Git pre-commit hook

You can automate the quality checks by registering a Git pre-commit hook. This
will run the `check` task every time you commit.

~~~~ bash
mise generate git-pre-commit --write --task=check
~~~~
