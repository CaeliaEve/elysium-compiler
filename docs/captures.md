# Capture recovery

The development candidate pairs NESQL 0.45.0-semantic.13 with Compiler and
contracts 0.41.0-semantic.1, Source revision 41 and Catalog revision 41.
Local mechanism checks have passed; full-pack native acceptance still requires
the external game batch. The completed archive envelope remains revision 1.
NESQL also writes separate same-session Handler checkpoints. Those journals
are **not** Compiler inputs: the producer must validate native data again and
finish a new ordinary Source before compilation. `assemble` still rejects all
`writing` capture archives, including ones with some completed Handler units.
The original failed job and archive are preserved by a producer-side retry.
The new producer checkpoint envelope is revision 5, with semantic proof revision 3
and continuation proof revision 1, and binds Source revision
41 and presentation `semantic-content-v1` in addition to the producer, native
provenance and ordered handlers. The environment also records these capability
fields, so they participate in environment and runtime fingerprints. A revision
1/2/3/4 checkpoint or a different presentation cannot be replayed, even if every
file hash is valid. This does not change the completed capture envelope's
revision or authorize mixing historical `writing` archives.

Revision 5 retains the per-unit inventory resource classification. The producer
now wires native renderer/model/texture/shader authority and each unit's owned
Random boundary state into recovery. Such prefixes additionally require the
exact released path/digest/count in its process-local capability store. Missing,
evicted or cross-process capabilities cannot be replaced by JSON or hashes.
Replaying a shorter prefix retains that earlier unit's state rather than the
old terminal Random position. Local replay, corrupt/stale-prefix rejection and
resource-cleanup checks have passed; native game continuation remains to be verified.
Compiler assembly cannot authorize producer continuation and
still accepts only a complete ordinary capture envelope.

Semantic proof revision 3 adds ordered native capture dispositions. Handler
units distinguish captured entries, proven exclusions, identical duplicates,
domain references and emitted branches. Complete disposition events and their
count/hash chain are checked before sealing and rechecked against native data
on producer recovery. Failed or abandoned attempts do not reserve recipe IDs.
These events belong to the producer journal, not Source collections or Compiler
inputs. A revision 2 semantic proof lacks this evidence and cannot be resumed.
For selected Magic recipes, the producer checkpoint seed also includes the
complete registered research dependency closure. Same-session recovery takes
a fresh native snapshot and checks research prerequisites, knowledge and
triggers before replay is accepted. This does not enable full-domain or
cross-build recovery, and does not make a checkpoint a valid Source.

Selected native structure handlers now journal their component/tier references
and owned construction geometry in the same Handler unit. Recovery rechecks the
native scan, controller/provider binding, parameters, descriptions, definition
cells, actual block states and typed tile NBT before replay. Geometry proof
normalizes renderer-dependent model attachments and build IDs, but retains each
occupied cell and native orientation; asset bytes remain independently verified
by the checkpoint blob manifest. Neither graphics normalization nor a ready-empty
scan permits missing geometry, unsealed units, cross-session or cross-version reuse.

Compiler 0.14.3 also accepts NESQL 0.15.4's explicit `scope: recipes` exports.
That scope must match the environment, name explicit handlers and a world, and
remain a `selection` Source. Runtime comparison excludes this request field;
the complete environment and selection hashes still include it. This supports
bounded live pipeline validation without claiming full registry coverage.

NESQL 0.15.3 writes closed Source files to `nesql/captures/<job-id>` while the
production export is running. Compiler 0.14.2 can assemble a completed archive
without loading Minecraft. Source revision 15 and Catalog revision 14 are used; this transport
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
From NESQL 0.42.2 and Compiler 0.37.2, there is no separate file-count cap.
Individual blobs and the Source manifest remain limited to 64 MiB; the capture
v1 manifest permits 129 MiB because it embeds the descriptors twice (the outer
`files` plus `source.files`), with 1 MiB of envelope headroom. The embedded Source
is checked against its own 64 MiB budget before assembly. Native capture also
counts canonical descriptor bytes as they arrive and refuses a descriptor list
that cannot fit a Source; sealing checks the exact complete Source size.
The total data cap remains 128 GiB, with existing Source collection limits.
This changes capacity policy only: capture revision 1, Source revision 37,
Catalog revision 36, identities and provenance rules are unchanged. Older
compilers retain their 100,000-file / 64 MiB capture limits; use the paired release.

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
