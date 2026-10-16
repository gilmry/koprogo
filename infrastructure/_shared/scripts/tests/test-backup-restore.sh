#!/usr/bin/env bash
# Automated backup/restore round-trip test (#354 WP-E2, Phase 3).
#
# Mirrors the key model of roles/backup/templates/backup-encrypted.sh.j2:
# the server only holds the PUBLIC key (--recipient-file) and cannot decrypt;
# restores happen off-host with the offline private key. A regression there
# (compression flag mismatch, swapped recipient, private key back on the
# host) is caught without needing a live production backup.
#
# Uses a fast ECC key (`--quick-generate-key ... default`); the algorithm is
# irrelevant to what this script verifies.
set -euo pipefail

WORKDIR=$(mktemp -d)
trap 'rm -rf "$WORKDIR"' EXIT

# Offline keyring: holds the private key (operator workstation / vault).
GNUPGHOME="$WORKDIR/offline"
export GNUPGHOME
mkdir -m 700 "$GNUPGHOME"
echo "allow-loopback-pinentry" > "$GNUPGHOME/gpg-agent.conf"

RECIPIENT="backup-restore-test@koprogo.local"

gpg --batch --yes --passphrase '' --pinentry-mode loopback \
  --quick-generate-key "$RECIPIENT" default default never
gpg --batch --armor --export "$RECIPIENT" > "$WORKDIR/backup-pubkey.asc"

echo "koprogo backup/restore test payload $(date +%s)" > "$WORKDIR/plain.sql"

# "Server" side: empty keyring, public key file only - as deployed by the role.
HOST_GNUPGHOME="$WORKDIR/host"
mkdir -m 700 "$HOST_GNUPGHOME"
gzip -9 < "$WORKDIR/plain.sql" \
  | GNUPGHOME="$HOST_GNUPGHOME" gpg --batch --encrypt \
      --recipient-file "$WORKDIR/backup-pubkey.asc" \
      --compress-algo none --output "$WORKDIR/backup.sql.gz.gpg"

# @security - the server keyring must not be able to decrypt its own backups.
if GNUPGHOME="$HOST_GNUPGHOME" gpg --batch --decrypt --quiet \
    "$WORKDIR/backup.sql.gz.gpg" >/dev/null 2>&1; then
  echo "FAIL @security: backup is decryptable on the host" >&2
  exit 1
else
  echo "PASS @security: host cannot decrypt (no private key on the server)"
fi

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
