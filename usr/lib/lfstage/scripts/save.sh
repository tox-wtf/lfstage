#!/bin/bash
# Script to save the stage file. Also includes logic for stripping.
# TODO: Consider moving the stripping logic to its own file instead :shrug:
#
# shellcheck disable=2164

# Sanity checks
if [[ "$LFS" != "/var/lib/lfstage/mount" ]]; then
    die "\$LFS isn't properly set" 33
fi

cd "$LFS"

TMPDIR="/tmp/lfstage/$LFSTAGE_PROFILE"

# Mass strip
if [ -f "$TMPDIR/strip" ]; then
    msg "Mass stripping..."
    find . -type f -executable -exec file {} + |
        grep 'not stripped' |
        cut -d: -f1         |
        while read -r file; do
            echo "lfstage: stripping $file"
            strip --strip-unneeded "$file"
        done
    msg "Stripped!"
fi

# Save the stage file
msg "Saving stage file..."
STAGEFILE="$(cat "/tmp/lfstage/$LFSTAGE_PROFILE/stagefilename")"
XZ_OPT=-9e tar cJpf "$STAGEFILE" .

# Add a convenience symlink
BASENAME="$(basename "$STAGEFILE")"
ln -sfv "../profiles/$LFSTAGE_PROFILE/stages/$BASENAME" "/var/cache/lfstage/stages/$BASENAME"

# Finalize
cd /
if mount | grep "$LFS"; then
    umount -Rv  "$LFS"
fi
