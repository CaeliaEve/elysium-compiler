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

The initial actual-instance sample (2026-09-28) used five pinned archives:
Minecraft, IC2, GregTech, Thaumcraft and Forestry. It produced 21,375 candidates
and 10,845 blobs; every blob's length and digest was verified independently. This
is an offline sample, not evidence of complete resource coverage, resource-pack
precedence, a successful game export or a new full GTNH Catalog.
