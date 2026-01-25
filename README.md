<h1 align="center">
LFStage
</h1>
<h2 align="center">
LFS stage file generator
</h2>

## Status - 75%
Basic documentation exists in the form of man pages. The ABI should now be
stable, with any new additions remaining backwards-compatible. Some refactoring
is probably still warranted. UX has been improved, but UX and especially UI
could still use some work. Support for profile imports and exports looks solid.

I plan to write HTML docs with `mdbook` that run through the entire process of
creating a profile, from setting up the git repo to executing the build, and
then using the stage file afterwards. I'd like to add optional per-profile
requirements scripts (these should be backwards compatible). I need to address
miscellaneous comment todos. Finally, I'll set up github actions to aid in
long-term maintenance.

## Introduction
LFStage builds [stage files](https://wiki.gentoo.org/wiki/Stage_file) for
[Linux From Scratch](https://www.linuxfromscratch.org/). However, it's probably
agnostic enough to build stage files for other systems.

### Features
- Profiles
- Mass stripping
- Configuration
- Logging

### A high-level overview
The central component of LFStage is the *profile*. A profile defines sources and
build instructions.

LFStage handles downloading those sources and executing the scripts. LFStage
also handles certain boilerplate tasks internally, including cleaning the build
environment before and after a build, setting up a minimal environment,
stripping binaries, and saving the stage file.

## Installation
To download and install LFStage run the following commands:
```bash
git clone https://github.com/tox-wtf/lfstage.git
cd lfstage
git submodule update

# TODO: Add ./configure
make
sudo make install
```

If you'd like to examine the structure of LFStage, execute the following
commands as well:
```bash
make DESTDIR="$PWD/DESTDIR" install
tree DESTDIR
```

<!-- TODO: If there's demand for it, use POSIX-compliant sh for internal scripts
-->
## Dependencies
- Required
    - Glibc (musl might also work, but I haven't tested it)
- Build
    - Rust
- Runtime
    - Bash
    - Git (for importing profiles)
    - Curl (for importing profiles)
    - LFS requirements

<!--
 TODO: Cache results of reqs.sh, maybe in /tmp/lfstage/reqs.cache, so it's
not run more than once per boot.

Also consider adding support for per-profile `reqs.sh`'s. If I do this, have a
reqs.env defining the basic functions to reduce boilerplate for profile authors.

Yeah I probably should add per-profile `reqs.sh` support. It's nice to be able
to check you meet requirements before running `build`, and it would allow
profile authors a standard way to define profile requirements.
-->
LFStage will run `/usr/lib/lfstage/scripts/reqs.sh` before building to ensure
general requirements are met.

## Basic usage
Let's say you wanted to build the profile `x86_64-glibc-tox-stage2`:

```bash
# First, import it
sudo lfstage import https://github.com/tox-wtf/x86_64-glibc-tox-stage2-lfstage.git

# Download the sources and build the stage file
# Note this command takes a long time -- ~30 minutes on my system
sudo lfstage build x86_64-glibc-tox-stage2

# View the completed build
tar tf "$(command ls -1t /var/cache/lfstage/profiles/x86_64-glibc-tox-stage2/stages/* | head -1)"

# View the build log
less -R /var/log/lfstage/lfstage.log
```

<!--
 TODO: Add `./patches/`. Explain that the patches should be applied with `git
apply patches/<patch>`.

Ideas:
- Compression algorithm patches
-->

## Profiles
Here are some profiles for LFStage. They could be considered reference
implementations:
- [to](https://github.com/toxikuu/to-lfstage): This profile is used as a build
environment by by [to](https://github.com/toxikuu/to)
- [x86_64-glibc-tox-stage2](https://github.com/toxikuu/x86_64-glibc-tox-stage2-lfstage):
This is the original LFStage profile. It's a minimal stage 2 LFS system, an
isolated starting point from which Chapter 8 of the LFS book can be executed.
- [testing](https://github.com/toxikuu/testing-lfstage): This is a minimal testing
profile.

## Todos
- [ ] Address all comment todos
- [x] Make `lfstage build` run the download logic if any sources are missing
- [ ] Add more subcommands
    - [x] `lfstage view` should list the available profiles
        - [x] `lfstave view <profile>` should list details about a profile
            - [ ] Add a system for profile metadata containing information like
              a description, author, notes, etc.
    - [ ] `lfstage reqs <profile>` assuming I add per-profile reqs.sh support
    - [x] `lfstage import path/to/<profile>.tar.xz`
        - [x] Support `lfstage import <https://git.repo.git>`
    - [x] `lfstage export <profile> <optional-destination>.tar.xz`
- [x] Move the profiles included into their own repositories
    - [x] Decide on a format for repos (\<profile\>-lfstage)
- [x] Add a profile struct
- [ ] Write documentation
    - [x] man
    - [ ] docs
    - [ ] code
- [ ] `./configure` script, supporting standard variables
- [x] More configuration options
    - [x] ~~Jobs~~ Makeflags
- [ ] GitHub actions
    - [ ] Formatting
        - [ ] Trimming white space
        - [ ] Rustfmt
    - [ ] Test
        - [ ] Lint
        - [ ] Nextest
        - [ ] Audit
    - [ ] Release
        - [ ] Changelog
        - [ ] Build stage file
        - [ ] Stage file
