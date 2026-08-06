#!/bin/sh
# Guest-side: build the data set that the backup tests run against.
#
# Not just random bytes. Each file is there to make one assertion possible:
#
#   big-*.bin        volume, and incompressible so pool size is meaningful
#   twin-a/b.bin     byte-identical → intra-backup deduplication
#   mutable.txt      rewritten between backup #1 and #2 → incremental
#   xattr.txt        user.* extended attribute (Linux and FreeBSD)
#   acl.txt          POSIX ACL (Linux only)
#   link-to-*        symlink, must survive the round trip
#   sparse.bin       sparse file
#   "space and é"    non-ASCII characters and spaces in the name
#   skip.nobackup    matches the share exclude → must NOT appear in the manifest
#
# Runs on both Linux and FreeBSD, so: /bin/sh rather than bash, and every tool
# that differs between the two is probed rather than assumed.
#
# Usage: gen-testdata.sh <directory>

set -eu

# FreeBSD has sha256(1) and no sha256sum(1); Linux has the reverse. Their output
# also differs: `sha256 -r` separates hash and name with ONE space, GNU
# sha256sum with two — and `sha256sum -c` silently skips lines it cannot parse,
# so a naive port would "verify" nothing. Normalise to the GNU format, which is
# what both `sha256sum -c` and `sha256 -c` accept.
if command -v sha256sum >/dev/null 2>&1; then
    checksum() { sha256sum "$@"; }
else
    checksum() { sha256 -r "$@" | sed 's/ /  /'; }
fi

DIR="${1:?usage: gen-testdata.sh <directory>}"
BIG_MB="${BIG_MB:-64}"

rm -rf "${DIR}"
mkdir -p "${DIR}/nested/deeper"

echo "generating in ${DIR} (large files: ${BIG_MB} MiB each)"

for n in 1 2 3; do
    dd if=/dev/urandom of="${DIR}/big-${n}.bin" bs=1M count="${BIG_MB}" status=none
done

# Identical content in two places: the pool must store the chunks once.
dd if=/dev/urandom of="${DIR}/twin-a.bin" bs=1M count=8 status=none
cp "${DIR}/twin-a.bin" "${DIR}/nested/twin-b.bin"

printf 'version 1\n' > "${DIR}/mutable.txt"

printf 'has an extended attribute\n' > "${DIR}/xattr.txt"
if command -v setfattr >/dev/null 2>&1; then
    setfattr -n user.woodstock -v e2e "${DIR}/xattr.txt"
elif command -v setextattr >/dev/null 2>&1; then
    setextattr user woodstock e2e "${DIR}/xattr.txt"
fi

# POSIX ACLs are Linux-only here: FreeBSD UFS needs to be mounted with `acls`,
# which the stock cloud image is not. The failure is tolerated so the file still
# exists everywhere and the platform-specific assertions decide what to expect.
printf 'has an ACL\n' > "${DIR}/acl.txt"
if command -v setfacl >/dev/null 2>&1; then
    setfacl -m u:nobody:r-- "${DIR}/acl.txt" 2>/dev/null \
        || echo "note: ACLs not supported on this filesystem, skipping"
fi

ln -sf big-1.bin "${DIR}/link-to-big-1"

truncate -s 33554432 "${DIR}/sparse.bin"   # 32 MiB; -s accepts plain bytes on both platforms

printf 'accents and spaces\n' > "${DIR}/space and éàü.txt"

printf 'this must be excluded\n' > "${DIR}/skip.nobackup"

printf 'deep file\n' > "${DIR}/nested/deeper/leaf.txt"

# A manifest of what was written, so the restore test can diff against it
# without depending on the harness remembering the layout. *.nobackup is left
# out on purpose: the share excludes it, so a restore must not bring it back and
# listing it here would turn correct behaviour into a checksum failure.
#
# No -print0/xargs -0 here: BSD xargs has no -0, and none of the generated names
# contain a newline.
( cd "${DIR}" \
    && find . -type f ! -name checksums.sha256 ! -name '*.nobackup' \
    | sort \
    | while IFS= read -r f; do checksum "${f}"; done > checksums.sha256 )

echo "generated:"
du -sh "${DIR}"
wc -l < "${DIR}/checksums.sha256" | xargs echo "files checksummed:"
