LFSTAGE-PROFILE(5)


# NAME

lfstage-profile - stage file specification for _lfstage_(1)


# SYNOPSIS

The structure of a generic profile, as viewed with *tree -F*:

```
./
├── envs/
│   ├── base.env
│   └── build.env
├── LICENSE
├── README.md
├── scripts/
│   ├── 05-setup.sh*
│   ├── 10-stage1.sh*
│   ├── 15-stage2.sh*
│   ├── 20-stage3.sh*
│   └── libexec/
│       └── as_chroot.sh
└── sources
```


# FILES

*sources*

An lfstage profile must contain a *sources* file. See _lfstage-sources_(5) for
syntax information. The sources file is parsed, and its contents are downloaded
in parallel to */var/cache/lfstage/profiles/<profile>/sources/*. These later get
copied to */var/lib/lfstage/mount/sources/*.

*envs/*

The envs directory contains environments for the build. The only required
environment is base.env, which might look something like so:

```
# envs/base.env

export LFS_TGT="x86_64-lfs-linux-gnu"
export C{,XX}FLAGS="-O2 -pipe"
export PATH="$LFS/tools/bin:/usr/bin:/usr/sbin"
export CONFIG_SITE="$LFS/usr/share/config.site"
export LDFLAGS="-Wl,-O1 -Wl,--as-needed"
set +h -euo pipefail
```

To use an environment in your chroot, simply copy it over before chrooting. The
*ENVS* variable, among others, is set by _lfstage_(1).

	cp -vf "$ENVS/build.env" "$LFS/build.env"

And ensure your chroot script sources it.

*scripts/*

The scripts directory contains the actual build logic. For _lfstage_(1) to
register a script, it must start with two digits, followed by a dash, as shown,
and it *must be executable*. Though not required, scripts that call other scripts
should be placed in a subdirectory, such as *libexec/*.

It is recommended to partition your builds into separate stages, though you can
organize your profile however you like.

*README.md*

This should contain a brief description and general overview of the profile.

*LICENSE*

The license under which the profile is distributed.


# ENVIRONMENT

The _lfstage_(1) program exposes several environment variables for convenience,
with sane defaults chosen for compatibility. More may be provided in the future.

```
# /usr/lib/lfstage/envs/internal.env

TZ=UTC
LFS=/var/lib/lfstage/mount
TERM=xterm-256color
LC_ALL=POSIX

ENVS=/path/to/profile/envs # populated at runtime
```
; TODO: Add SCRIPTS environment variable


# STANDARDS

A git repository for an lfstage profile should follow the scheme of
*<name>-lfstage*. This keeps the repository distinct from other repositories
with similar names, makes its purpose clear, and simplifies the parsing of
profile imports.


# CONVENTIONS

Files should have their paths relative to the profile root commented on the
first line, or second if a shebang occupies the first line.

The usage of *libexec/* for auxiliary scripts isn't required, but follows
convention. This is generally used for chrooting, but could theoretically store
scripts for individual packages, for instance.


# EXAMPLES

; TODO: Update this whenever API changes. <- when the SCRIPTS variable gets
; exposed
Snapshots of two profiles may be viewed at the following links:
. https://github.com/tox-wtf/to-lfstage/tree/99f4231eb670a02066b3c751d235e040ef2fff41
. https://github.com/tox-wtf/x86_64-glibc-tox-stage2-lfstage/tree/a1562f556926b53c827962893704df79f7dc22b9


# NOTES

*Internal profiles and scripts*

Astute readers might have noticed some semblance between the structure of
profiles defined in */var/lib/lfstage/profiles/<profile>/* and lfstage's
own hierarchy, defined in */usr/lib/lfstage/*. This is by design in an
attempt to make things more intuitive.

Since the internal environment is sourced first, any environment variables it
sets may be overwritten by your own *base.env*.

The _lfstage_(1) program automates some boilerplate via internal scripts. These
scripts may be viewed at */usr/lib/lfstage/scripts/*. A brief synopsis of each
follows:

*clean.sh*

Cleans */var/lib/lfstage/mount*, recursively unmounting and removing any stale
files.

*reqs.sh*

Ensures the host system meets build requirements.

*save.sh*

Strips all binaries, saves the stage file with xz compression, and cleans up.
This script is run after all profile-defined scripts are run.

*{import,export}.sh*

Helper scripts for importing and exporting profiles. These are not a part of the
build process.
*testing.sh*

A script used for testing, subject to removal.

I suggest reviewing the internal scripts and environment(s), as well as one of
the linked example profiles before writing your own.


# SEE ALSO

_lfstage-sources_(5) _lfstage_(1)


# AUTHORS

Created and maintained by Tox <toxtek@proton.me>. Upstream is located at
https://github.com/tox-wtf/lfstage.git.
