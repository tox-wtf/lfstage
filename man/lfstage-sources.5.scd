LFSTAGE-SOURCES(5)


# NAME

lfstage-sources - the sources registry for _lfstage_(1)


# DESCRIPTION

An _lfstage_(1) sources file is a new-line delimited list of URLs, optionally
with a destination. Empty lines and lines starting with '#' are ignored.
Repositories and directories are not currently supported.

To specify a destination, append ' -> destination' to the line.

The sources file should be placed at the root of the profile. See
_lfstage-profile_(5) for structure.


# EXAMPLES

A snapshot of the sources for the to profile may be viewed at the following
link:
- https://github.com/tox-wtf/to-lfstage/blob/99f4231eb670a02066b3c751d235e040ef2fff41/sources

A brief example is also provided below:

```
# sources

https://ftpmirror.gnu.org/gnu/autoconf/autoconf-2.72.tar.xz -> ac-2.72.txz
https://ftpmirror.gnu.org/gnu/automake/automake-1.18.tar.xz -> am-1.18.txz
https://ftpmirror.gnu.org/gnu/bash/bash-5.3-rc1.tar.gz
https://sourceware.org/pub/binutils/releases/binutils-2.44.tar.xz
https://ftpmirror.gnu.org/gnu/bison/bison-3.8.2.tar.xz
```


# SEE ALSO

_lfstage-profile_(5) _lfstage_(1)


# AUTHORS

Created and maintained by Tox <toxtek@proton.me>. Upstream is located at
https://github.com/tox-wtf/lfstage.git.
