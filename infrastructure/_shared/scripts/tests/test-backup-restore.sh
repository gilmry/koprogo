#!/usr/bin/env bash
# Automated backup/restore round-trip test (#354 WP-E2, Phase 3).
#
# Mirrors the encrypt -> compress -> decrypt -> integrity-check logic of
# test_restore() in roles/backup/templates/backup-encrypted.sh.j2, so a
# regression there (e.g. a compression flag mismatch, or a swapped
# recipient) is caught without needing a live production backup.
#
# Uses a fast ECC key (`--quick-generate-key ... default`) rather than the
# role's actual RSA-4096 (see roles/backup/templates/gpg-key-params.j2) —
# key algorithm/size is irrelevant to what this script verifies (the
# encrypt/decrypt/corruption-detection round trip), and RSA-4096 key
# generation is exercised separately by the backup role's molecule scenario
# (roles/backup/molecule/default/), where haveged supplies entropy.
set -euo pipefail

WORKDIR=$(mktemp -d)
trap 'rm -rf "$WORKDIR"' EXIT

GNUPGHOME="$WORKDIR/gnupg"
export GNUPGHOME
mkdir -m 700 "$GNUPGHOME"
echo "allow-loopback-pinentry" > "$GNUPGHOME/gpg-agent.conf"

RECIPIENT="backup-restore-test@koprogo.local"

gpg --batch --yes --passphrase '' --pinentry-mode loopback \
  --quick-generate-key "$RECIPIENT" default default never

echo "koprogo backup/restore test payload $(date +%s)" > "$WORKDIR/plain.sql"

gzip -9 < "$WORKDIR/plain.sql" \
  | gpg --encrypt --recipient "$RECIPIENT" --trust-model always \
      --compress-algo none --output "$WORKDIR/backup.sql.gz.gpg"

# @happy — a valid encrypted backup must decrypt and pass gunzip's integrity
# check, exactly like test_restore() expects.
if gpg --decrypt --quiet "$WORKDIR/backup.sql.gz.gpg" 2>/dev/null \
    | gunzip -t 2>/dev/null; then
  echo "PASS @happy: valid encrypted backup restores cleanly"
else
  echo "FAIL @happy: a valid backup failed the restore test" >&2
  exit 1
fi

# @negative — a corrupted backup must be DETECTED, not silently accepted as
# restorable. A restore test that always reports success is worse than no
# test at all.
cp "$WORKDIR/backup.sql.gz.gpg" "$WORKDIR/corrupt.sql.gz.gpg"
printf '\x00\x00\x00\x00' \
  | dd of="$WORKDIR/corrupt.sql.gz.gpg" bs=1 seek=20 conv=notrunc status=none

if gpg --decrypt --quiet "$WORKDIR/corrupt.sql.gz.gpg" 2>/dev/null \
    | gunzip -t 2>/dev/null; then
  echo "FAIL @negative: corrupted backup was NOT detected by the restore test" >&2
  exit 1
else
  echo "PASS @negative: corrupted backup is correctly rejected"
fi

# @edge — an empty/zero-byte backup file (e.g. pg_dump failed silently and
# produced nothing) must also be rejected, not treated as a valid empty
# restore.
: > "$WORKDIR/empty.sql.gz.gpg"
if gpg --decrypt --quiet "$WORKDIR/empty.sql.gz.gpg" 2>/dev/null \
    | gunzip -t 2>/dev/null; then
  echo "FAIL @edge: empty backup file was NOT detected as invalid" >&2
  exit 1
else
  echo "PASS @edge: empty backup file is correctly rejected"
fi

echo "All backup/restore GPG round-trip checks passed"
