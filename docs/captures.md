# Capture recovery

NESQL 0.15.3 writes closed Source files to `nesql/captures/<job-id>` while the
production export is running. Compiler 0.14.2 can assemble a completed archive
without loading Minecraft. Source and Catalog remain revision 14; this transport
envelope is `elysium.capture`, revision 1.

## Files and completion

- `blobs/<sha256>` holds exact closed Source bytes, deduplicated by content.
- `parts/<sha256-of-logical-path>.json` is an atomic descriptor receipt for each
  file. A receipt is written after verifying and flushing the blob.
- `manifest.json` initially has `state: writing`, empty `files`, provenance and
  the serialized request. It is not rewritten for every texture. Job progress
  samples the number of durable files every 64 receipts; this count can lag.
- After all native capture, native resource cleanup, environment verification,
  record sorting and the final session guard, NESQL verifies all blobs/receipts
  and atomically writes `state: complete`, the exact `files` list, and the native
  `source` manifest. The job's `fragments` receipt pins this manifest by SHA-256
  and byte length. A later Source publication failure does not undo the archive.

The envelope `id` is SHA-256 of canonical JSON with `id` omitted. Provenance
binds the environment, runtime, session and selection. These hashes detect
changed/mixed data relative to the trusted native receipt; they are not digital
signatures. Keep the original receipt and use its pinned hash, rather than
accepting a newly calculated hash after unexpected file changes.

Native Source files and archive blobs may be hard links on the same filesystem.
Treat both as immutable: changing either corrupts both. If links are unavailable,
NESQL copies and verifies the bytes. Recovery creates separate output files.
Existing capture directories are never reused or overwritten.

## Offline assembly

```powershell
elysium-compiler assemble --input <capture-directory> --sha256 <job.fragments.sha256> --output <new-source-directory>
elysium-compiler inspect --input <new-source-directory>
elysium-compiler compile --input <new-source-directory> --output <new-catalog-root>
elysium-compiler check --input <new-catalog-root>
```

`assemble` requires a complete native export envelope, its original manifest
digest, an exact descriptor set, all referenced blobs, matching provenance and
the ordinary Source domain/identity/reference/pixel checks. Missing files,
damage, incomplete captures and diagnostic requests fail with a nonzero exit.
Individual blobs and the manifest are limited to 64 MiB; the file set is limited
to 100,000 entries and 128 GiB in total. Existing Source collection limits still
apply.

Input and output must be disjoint, with no symbolic links, junctions or `..`
components. The output parent uses an exclusive `.source.lock`. Files are copied
and verified in a temporary directory, then renamed into place only after all
checks pass. On reuse, both archive content and the existing Source are checked;
a different or damaged output is rejected and left untouched.

The returned JSON reports the original Source `id`, `environment`, `scope`,
capture identity/digest, output location, file/byte counts and `reused`. Recovery
does not change selected snapshots into full snapshots. It does not alter the
failed game job or register a successful result in the game's export listing.
Keep the assembly receipt beside the original job when handing off recovery.

## Current limits

This is recovery after capture completed, not arbitrary game range resume.
Records are currently emitted as final sorted shards near the end of capture;
an early failure retains closed assets and any completed shards, not a complete
replay of pending in-memory facts. The assembler deliberately refuses these
`writing` archives. It does not combine jobs, re-run native callbacks, skip
recipe ranges, infer unsupported semantics or replace icons with unverified JAR
resources. `parts` receipts are evidence for unfinished captures, not an
alternative route around the completion manifest.

Atomic file replacement and file flushing support interruption/restart recovery.
They do not provide a transactional guarantee against hardware or filesystem
failure; all recovery rechecks integrity. Full GTNH coverage and performance
still require the separately arranged live export acceptance.
