# ADR 0044: Document Storage Strategy

- **Status**: Accepted
- **Date**: 2025-10-28
- **Track**: Software

## Context

Document uploads were previously handled through a concrete `FileStorage` service that wrote
directly to the local filesystem (`./uploads`). Upcoming roadmap phases require the ability to
switch between local volumes, an in-cluster MinIO deployment, or a managed S3-compatible bucket.
Without a common abstraction the backend would have to be rewritten for every storage target and
tests would remain tightly coupled to the POSIX filesystem.

## Decision

We introduced a `StorageProvider` trait that defines the operations required by the document
workflow (`save_file`, `read_file`, `delete_file`, `file_exists`). The existing `FileStorage`
implementation now implements this trait and is injected as an `Arc<dyn StorageProvider>` into the
`DocumentUseCases`. All code paths now rely on the abstraction, so swapping the backing store only
requires providing another implementation at composition time.

In addition to the filesystem backend, we ship an `S3Storage` provider that targets MinIO or any
S3-compatible bucket. The deployment is controlled through `STORAGE_PROVIDER` (`local`, `s3`,
`minio`) and a set of `S3_*` environment variables (bucket, credentials, endpoint, region, prefix).
When the value is `local`, the application continues to use the original directory-based storage.

For Phase 1 we keep the local filesystem backend as the default because:

1. It is already deployed and well understood by the team.
2. It has zero external cost and does not require additional infrastructure.
3. It satisfies GDPR constraints when coupled with encrypted volumes and backups from adjacent
   roadmap items.

## Consequences

- The backend is now decoupled from a specific storage technology; additional providers can be added
  independently of the application layer.
- Unit tests can remain lightweight by reusing the in-memory/local implementation through the trait
  without further changes.
- Configuration remains backwards compatible: `STORAGE_PROVIDER` defaults to `local` and the
  existing `UPLOAD_DIR` variable keeps controlling the filesystem path. Switching to MinIO/S3 only
  requires setting the appropriate `S3_*` variables.
- Any new storage provider must implement the `StorageProvider` trait, ensuring consistent behaviour
  (path sanitation, error semantics) across backends.

## Next Steps

- Add integration tests that run against a MinIO container to validate multi-provider behaviour.
- Extend observability (metrics, structured errors) for storage operations across providers.
- Evaluate encryption-at-rest or KMS integration for S3 buckets to complement infrastructure
  security controls.
