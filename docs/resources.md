# Local resource candidates

`elysium-compiler resources --input <input.json> --output <new-directory>`
reads pinned ZIP/JAR files without starting Minecraft or executing mod classes.
It returns a portable resource inventory, **not** a Source or Catalog. The
existing Source and Catalog contracts are unchanged.

```json
{
  "format": "elysium.resources",
  "revision": 1,
  "archives": [
    {
      "key": "base.jar",
      "path": "relative-to-input/base.jar",
      "bytes": 12345,
      "sha256": "<64 lowercase hex characters>"
    }
  ]
}
```

Each key is a unique portable identifier supplied by the caller. File paths may
be absolute or relative to the input manifest; parent segments, links and Windows
junctions are refused. Paths locate inputs but do not appear in the resulting
identity. The caller must obtain the correct loaded archives from the actual
instance: directory presence alone does not prove a mod or resource pack is active.

The importer snapshots one archive at a time, checking its byte count and SHA256
before parsing that same private copy. It retains exact PNG, `.png.mcmeta`, `.lang`
and language JSON bytes under `assets/<namespace>/textures/` and
`assets/<namespace>/lang/`. It does not decode images, execute classes, interpret
recipes, copy arbitrary root files, or import model/audio definitions. Logical
resource paths preserve case and Unicode; they are never materialized as disk
paths. Only digest-named blobs are written.

`manifest.json` contains:

- `archives`: keys, sizes and digests, sorted by key, without physical paths.
- `entries`: logical path, archive key, size, SHA256 and `blobs/<digest>` path.
  All candidates from different archives survive, even at the same logical path.
- `resolution: "unverified"`: input order cannot select a winner. A later native
  game observation must prove the resource binding and its byte digest.
- `id`: SHA256 of the compact JSON representation with sorted object keys,
  excluding `id`. Entries are sorted by path then archive key.

Identical resource bytes share a blob. Relocating the inputs or reordering the
archive list produces byte-identical output. PNG bytes do not by themselves prove
the rendered item pixels: tint, layers, animation, atlas transforms and custom
renderers still require their native rendering semantics.

The whole inventory publishes into a fresh directory after all inputs pass.
Existing destinations are refused. Import failures discard the temporary stage;
the parent may retain its `.resources.lock` coordination file. Concurrent imports
to the same parent are refused while its lock is held. This is a local CLI
publication lock, not protection against unrelated processes replacing paths.

Bounds are explicit: 4 MiB input manifest; 1–1024 archives; 2 GiB per archive and
32 GiB total compressed input; ordinary single-volume ZIPs with at most 60,000
entries, a 32 MiB central directory and 4096-byte names; 64 MiB per selected
resource; 500,000 candidates and 8 GiB total candidate bytes before deduplication.
ZIP64, split or prefixed ZIPs, duplicate central names, invalid paths, selected
links, encryption, CRC errors and size/digest mismatches fail the import. These
limits deliberately reject unsupported input instead of silently omitting it.
The serialized output manifest is limited to 128 MiB.

## Native byte resolution

```bash
elysium-compiler resolve --resources <inventory-directory> --report <native-check.json> --sha256 <report-sha256> --environment <environment-id> --output <new-bindings-directory>
```

NESQL 0.15.1 adds `start_check` domain `resources` for an explicit selection of
1–128 namespaced texture or language paths. It hashes streams returned by the
game resource manager and reads successful targets again before completing the
report. The ordinary environment fingerprint is checked before and after.
These diagnostics do not export Source records or verify rendered item pixels.

`resolve` requires the original complete report pinned by SHA256 and an expected
environment digest (SHA256 of canonical `report.environment`). Every selected
resource must have exactly one passed observation; stopped, failed, missing or
duplicate observations are refused. The report bytes and manifest identity are
verified before matching. A matching archive must appear by digest in the loaded
mod set or the enabled resource-pack files of that environment. Directory packs
and generated resources without an archive candidate remain unsupported by this
offline step; they cannot silently become static assets.

The result is a small `elysium.bindings` staging inventory with copied verified
blobs. It records world, environment, input inventory and report digests. Each
entry carries the native resource name, exact byte count and digest, and all
loaded origins with the same matching bytes. Multiple identical candidates do
not reveal which pack supplied the stream, so no arbitrary winner is claimed.
Matching is indexed by resource path. The original inventory is never modified.

Bindings are an intermediate resource artifact, not a second recipe protocol and
not a Source/Catalog. They are not accepted by `inspect` or `compile`. Connecting
them to capture fragments and substituting verified resources is subsequent work.
The existing consumer schema and frontend remain unchanged.

The initial actual-instance sample (2026-09-28) used five pinned archives:
Minecraft, IC2, GregTech, Thaumcraft and Forestry. It produced 21,375 candidates
and 10,845 blobs; every blob's length and digest was verified independently. This
is an offline sample, not evidence of complete resource coverage, resource-pack
precedence, a successful game export or a new full GTNH Catalog.

## Compare direct resources with a formal Source

Compiler 0.14.1 accepts three optional flags together:

```bash
elysium-compiler resolve --resources <inventory> --report <check.json> --sha256 <check-sha256> --environment <check-environment-id> --source <native-source> --capture <export-job.json> --capture-sha256 <job-sha256> --output <fresh-bindings>
```

NESQL 0.15.2 supplies the required provenance. The job must be `succeeded`, without
an error or diagnostic request, and reference exactly this Source. The CLI checks
the full Source environment, all domain records/references/pixels, both pinned
receipts and their request digests. Runtime identity excludes only probes,
profile and handler selection from the full environment; session and explicit
world must agree. The full environment identities are retained separately and
need not agree across `data` diagnostics and `full` capture.

Only `resource` assets with an explicit matching PNG path, no timeline and no
interpolation are considered. The local PNG and native Source PNG must have
exactly equal dimensions and every decoded RGBA byte, including transparent RGB.
Different compression is allowed. A mismatch aborts publication; GL captures,
tints and animation sheets are not certified by name similarity. At least one
direct resource must match when these flags are supplied.

The bindings manifest additionally records `provenance` and `source`, with the
Source id, original scope, environment, capture receipt hash and verified asset
ids/pixel digests. A selection Source remains a selection; this check makes no
complete-coverage claim. Source and inventory files are read-only, and output
inside either input directory is refused. Source/Catalog revision and frontend
contracts are unchanged. This command proves the specific direct PNG bytes; it
does not yet replace game capture, assemble fragments or publish a Catalog.
