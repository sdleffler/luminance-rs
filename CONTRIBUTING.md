# Contributing

This document is the official contribution guide to luminance contributors must follow. It will be **greatly
appreciated** if you read it first before contributing. It will also prevent you from losing your time if you
open an issue / make a PR that doesn’t comply to this document.

<!-- vim-markdown-toc GFM -->

* [Support and donation](#support-and-donation)
* [Important resources](#important-resources)
  * [Repositories](#repositories)
  * [Documentation and help](#documentation-and-help)
* [Style conventions](#style-conventions)
  * [Coding](#coding)
* [How to make a change](#how-to-make-a-change)
  * [Process](#process)
  * [Git conventions](#git-conventions)
* [Release process](#release-process)
  * [Overall process](#overall-process)
  * [Changelogs update](#changelogs-update)
  * [Git tag](#git-tag)

<!-- vim-markdown-toc -->

# Support and donation

This project is a _free  and  open-source_ project. It has no financial motivation nor support. I
([@phaazon]) would like to make it very clear that:

- Sponsorship is not available. You cannot pay me to make me do things for you. That includes issues reports,
  features requests and such.
- If you still want to donate because you like the project and think I should be rewarded, you are free to
  give whatever you want.
- However, keep in mind that donating doesn’t unlock any privilege people who don’t donate wouldn’t already
  have. This is very important as it would bias priorities. Donations must remain anonymous.
- For this reason, no _sponsor badge_ will be shown, as it would distinguish people who donate from those
  who don’t. This is a _free and open-source_ project, everybody is welcome to contribute, with or without
  money.

# Important resources

## Repositories

- [luminance and its various crates](https://github.com/phaazon/luminance-rs).
- [The book](https://github.com/rust-tutorials/learn-luminance).

## Documentation and help

- [The online documentation](https://crates.io/crates/luminance).
- [The book](https://rust-tutorials.github.io/learn-luminance).
- [My blog](https://phaazon.net/blog), which contains various articles on luminance.

# Style conventions

## Coding

Coding conventions are enforced by `rustfmt`. You are expected to provide code that is formatted by `rustfmt`
with the top-level `rustfmt.toml` file.

Coding convention is enforced in the Continuous Integration pipeline. If you want your work to be
mergeable, format your code.

> Note: please do not format your code in a separate, standalone commit. This way of doing is
> considered as a bad practice as the commit will not contain _anything_ useful (but code
> reformatted). Please format all your commits. You can use various tools in your editor to do so,
> such as [rust-analyzer](https://github.com/rust-analyzer/rust-analyzer).

# How to make a change

## Process

The typical process is to base your work on the `master` branch. The `master` branch must always contain a stable
version of the project. It is possible to make changes by basing your work on other branches but the source
of truth is `master`. If you want to synchronize with other people on other branches, feel free to.

The process is:

1. (optional) Open an issue and discuss what you want to do. This is optional but highly recommended. If you
  don’t open an issue first and work on something that is not in the scope of the project, or already being
  made by someone else, you’ll be working for nothing.
2. Fork the project.
3. Create a branch starting from `master`. Name it according to the _Git Flow_ naming conventions:
  - `fix/your-bug-here`: if you’re fixing a bug, name your branch.
  - `feature/new-feature-here`: if you’re adding some work.
  - `doc/whatever`: if you’re updating documentation, comments, etc.
  - Free for anything else.
  - The special `release/*` branch is used to either back-port changes from newer versions to previous
    versions, or to release new versions by updating `Cargo.toml` files, changelogs, etc.
4. Make some commits!
5. Once you’re ready, open a Pull Request (PR) to merge your work on `master`. For instance, open a PR for
  `master <-- feature/something-new`.
6. (optional) Ask someone to review your code. This is optional as it will eventually be reviewed.
7. Discussion and peer-review.
8. Once the CI is all green, someone (likely me [@phaazon]) will merge your code and close your PR.
9. Feel free to delete your branch.

## Git conventions

It is **highly appreciated** if you can format your git messages such as:

> Starting with a uppercase letter, ending with a dot. #343
>
> The #343 after the dot is appreciated to link to issues. Feel free to add, like this message, more context
> and/or precision to your git message. You don’t have it in the first line of the commit message,
> but if you are fixing a bug or implementing a feature thas has an issue linked, please reference it.

I’m very strict on git messages as I use them to write `CHANGELOG.md` files. Don’t be surprised if I ask you
to edit a commit message. :)

# Release process

## Overall process

Releases occur at arbitrary rates. If something is judged urgent, it is most of the time released immediately
after being merged and tested. Sometimes, several issues are being fixed at the same time (spanning on a few
days at max). Those will be gathered inside a single update.

Feature requests might be delayed a bit to be packed together as well but eventually get released, even if
they’re small.

## Changelogs update

`CHANGELOG.md` files must be updated **before any release**. Especially, they must contain:

- The version of the release.
- The date of the release.
- How to migrate from a minor to the next major.
- Everything that a release has introduced, such as major, minor and patch changes.

## Git tag

Once a new release occur, a Git tag is created. Git tags are formatted regarding the project they refer to:

> <project-name>-X.Y.Z

Where `X` is the _major version_, `Y` is the _minor version_ and `Z` is the _patch version_. For instance
`luminance-0.37.1` is a valid Git tag, so is `luminance-derive-0.5.3`.

> <project-name>-X.Y.Z-rc.W

Where `W` is a number starting from `1` and incrementing. This format is for _release candidates_ and occur
when a new version (most of the time a major one) is to be released but more feedback is required.

Crates are pushed to [crates.io](https://crates.io) and tagged on Git.

[@phaazon]: https://github.com/phaazon
