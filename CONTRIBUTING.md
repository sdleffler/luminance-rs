# Contributing

This document is the official contribution guide to luminance contributors must follow. It will be **greatly
appreciated** if you read it first before contributing. It will also prevent you from losing your time if you
open an issue / make a PR that doesn’t comply to this document.

<!-- vim-markdown-toc GFM -->

* [Disclaimer and why this document](#disclaimer-and-why-this-document)
* [Important resources](#important-resources)
  * [Repositories](#repositories)
  * [Documentation and help](#documentation-and-help)
* [How to make a change](#how-to-make-a-change)
  * [Process](#process)
* [Release process](#release-process)
  * [Overall process](#overall-process)
  * [Changelogs update](#changelogs-update)
  * [Git tag](#git-tag)
* [Conventions](#conventions)
  * [Coding](#coding)
  * [Git](#git)
    * [Git message](#git-message)
    * [Commit atomicity](#commit-atomicity)
    * [Hygiene](#hygiene)
* [Support and donation](#support-and-donation)

<!-- vim-markdown-toc -->

# Disclaimer and why this document

People contributing is awesome. The more people contribute to Free & Open-Source software, the better the
world is to me. However, the more people contribute, the more work we have to do on our spare-time. Good
contributions are highly appreciated, especially if they thoroughly follow the conventions and guidelines of
each and every repository. However, bad contributions — that don’t follow this document, for instance — are
going to require me more work than was involved into making the actual change. So please read this document;
it’s not hard and the few rules here are easy to respect. Thank you!

# Important resources

## Repositories

- [luminance and its various crates](https://github.com/phaazon/luminance-rs).
- [The book](https://github.com/rust-tutorials/learn-luminance).

## Documentation and help

- [The online documentation](https://crates.io/crates/luminance).
- [The book](https://rust-tutorials.github.io/learn-luminance).
- [My blog](https://phaazon.net/blog), which contains various articles on luminance.

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

# Conventions

## Coding

Coding conventions are enforced by `rustfmt`. You are expected to provide code that is formatted by `rustfmt`
with the top-level `rustfmt.toml` file.

Coding convention is enforced in the Continuous Integration pipeline. If you want your work to be
mergeable, format your code.

> Note: please do not format your code in a separate, standalone commit. This way of doing is
> considered as a bad practice as the commit will not contain _anything_ useful (but code
> reformatted). Please format all your commits. You can use various tools in your editor to do so,
> such as [rust-analyzer](https://github.com/rust-analyzer/rust-analyzer).

## Git

### Git message

Please format your git messages like so:

> [project(s)] Starting with a uppercase letter, ending with a dot. #343
>
> The #343 after the dot is appreciated to link to issues. Feel free to add, like this message, more context
> and/or precision to your git message. You don’t have to put it in the first line of the commit message,
> but if you are fixing a bug or implementing a feature thas has an issue linked, please reference it, so
> that it is easier to generate changelogs when reading the git log.

The `[project(s)]` header is mandatory if your commit has changes in any of the crates handled by this repository.
If you make a change that is cross-crate, feel free to separate them with commas, like `[crate_a, crate_b]`. If
you make a change that touches all the crates, you can use `[all]`. If you change something that is not related
to a crate, like the front README, CONTRIBUTING file, CI setup, top-level `Cargo.toml`, etc., then you can omit
this header.

**I’m very strict on git messages as I use them to write `CHANGELOG.md` files. Don’t be surprised if I ask you
to edit a commit message. :)**

### Commit atomicity

Your commits should be as atomic as possible. That means that if you make a change that touches two different
crates, most of the time, you want two commits – for instance one commit for the backend crate and one commit
for the interface crate. There are exceptions, so this is not an absolute rule, but take some time thinking
about whether you should split your commits or not.

Commits which add a feature / fix a bug _and_ add tests at the same time are fine.

### Hygiene

When working on a fix or a feature, it’s very likely that you will periodically need to update your branch
with the `master` branch. **Do not use merge commits**, as your contributions will be refused if you have
merge commits in them. The only case where merge commits are accepted is when you work with someone else
and are required to merge another branch into your feature branch (and even then, it is even advised to
simply rebase). If you want to synchronize your branch with `master`, please use:

```
git switch <your_branch>
git fetch origin --prune
git rebase origin/master
```

On the same level, please squash / fixup your commits if you think they should be a single blob. This is a
subjective topic, so I won’t be _too picky_ about it, but if I judge that you should split a commit into
two or fixup two commits, please don’t take it too personal. :)

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

[@phaazon]: https://github.com/phaazon
