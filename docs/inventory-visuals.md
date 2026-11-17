# Inventory content programs (implementation, not release acceptance)

The working tree's `VisualData` contract includes bounded inventory content
programs. It does not accept source-provided JavaScript, GLSL, native GUI scenes
or recipe progress captures. The generated contracts and release revisions must
be updated together after all pending production types are collected.

`forestryInventory` is pinned to Forestry 4.10.17 and retains a native MathHelper
resource, a separate item/entity clock and ordered `forestryPart` draws. Bee
scenes contain ten parts plus an optional crown; butterfly scenes contain five.
The compiler checks both wing roles in their native draw positions and the
aggregate vertex budget. A part references one native mesh, its complete model
sampler and an optional typed `inventoryLightmap`. A dynamic wing cannot also
contain a baked Z angle.

`inventoryLightmap` is a separate constant-coordinate fixed-function stage. It
retains the uploaded 16 by 16 image, native texture matrix, current coordinate,
border and GL_CLAMP / CLAMP_TO_EDGE modes. A timeline, atlas substitution or
program-backed lightmap is rejected. Its asset reference participates in the
same typed closure and standalone lossless packing as native mip sheets.

Root and helper types participate in content IDs, typed reachability, orphan
checks, per-root resource/pixel budgets and catalog publication. A preview is
not also a sprite timeline. Native renderer/model/VBO authority lives in the
producer process; identical JSON or hashes do not provide that authority for
unfinished capture recovery.

Java/Rust/Node tests, contract generation, GLSL compilation and browser/native
pixel acceptance for these additions are still pending in the unified batch.
