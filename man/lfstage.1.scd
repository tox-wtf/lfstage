LFSTAGE(1)


# NAME

*lfstage* - generate LFS stage files


# SYNOPSIS

lfstage [subcommand] [flags...] _profile_


# DESCRIPTION

The *lfstage* program reads _lfstage-profile_(5) specifications and creates LFS
stage files from them.


# EXAMPLES

	*lfstage* import https://github.com/toxikuu/x86_64-glibc-tox-stage2-lfstage.git

	*lfstage* build x86_64-glibc-tox-stage2


# ENVIRONMENT

The *lfstage* program accepts the *LOG_LEVEL* environment variable to control
verbosity. It may be one of:
. trace (default)
. debug
. info
. warn
. error
. off


# SEE ALSO

_lfstage-profile_(5)


# AUTHORS

Created and maintained by Tox <toxtek@proton.me>. Upstream is located at
https://github.com/tox-wtf/lfstage.git.
