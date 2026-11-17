# EnderStorage world-state contract (development)

The two native CustomDiagram overview routes now use a dedicated state adapter. It never calls
diagram suppliers, sends refresh packets, creates storage through `getStorage`, calls `loadFromTag`
on live storage, or saves inventories. `EnderItemStorage.saveToTag` can resize and dirty the live
inventory, so it is deliberately excluded.

`enderStorageDefinition` declares an owned snapshot at one integrated-server tick, limited to
the native global owner and the current player's exact display-name owner. This matches the
overview's owner choices; it does not include other players' personal storage. A complete local
server manager is mandatory. Partial remote-client observations cannot be represented as a
complete snapshot or as an empty world. The existing export service supports local worlds only.

The server tick queue pins the integrated-server identity and reuses bounded queue deadlines,
safe cancellation and original-failure handling. The snapshot merges unloaded `saveTag` entries
with loaded `storageMap` objects; loaded contents override stale saved contents. Reads occur on
the owning server thread. Arrays, stacks, fluids, NBT and output rows are owned copies. Entry,
frequency, NBT, depth, leaf and total byte limits are checked. No state is read through a getter
which lazily creates or aligns inventory.

State rows preserve literal owner names, original 12-bit frequencies and native display colour
order (reversed for tanks), stored size index, every empty slot, exact amounts and component NBT.
Persisted rows also retain typed original NBT. Loaded rows retain their current array size without
calling the native alignment routine. Persisted sizes are raw stored sizes, not a prediction of
the size after future loading. Missing persisted registry identities or malformed/repeated slots
fail explicitly. The native empty persisted-tank sentinel is water at zero amount; it is not
invented visible fluid. Unallocated frequencies have the native empty default and do not become
thousands of fabricated records.

The definition preserves native block/pouch query rules, personal locking item and the four action
keys. Refresh actions require a new capture; the webpage does not send game packets or claim that
reloading a Catalog refreshes the world. Page content is a snapshot at `capturedTick`. Sealing
checks world/player/manager/callback/configuration/locale ownership, rather than silently updating
the content from a later tick. Reopening a fresh cursor creates a fresh snapshot: completed units
from another snapshot cannot be combined without a separate validity proof. Existing recovery
will reject changed snapshot identities; no cross-job or cross-session reuse is claimed.

Rust independently validates complete row order/counts, origin, actions, query rules, identities,
layer policy, inventory size/emptiness and fluid capacity. It checks derived quantities, metadata
and NBT against the raw persisted tag, including native numeric `Quantity` conversion. Rehashing
malformed semantics does not make them valid. The generated shared TypeScript query repeats the
structural/source/raw-tag checks, indexes one selected group within a byte budget, and hydrates
only visible inventory slots, selected fluids and the locking item. `/ender-storage` supports
source-order paging, owner/frequency/content filtering, native device queries, deep links and
the same online/offline reader.

Source 40 / Catalog 39 with generated package 0.40.0-semantic.31 remains a development contract.
Local evidence uses actual installed EnderStorage 1.7.7 storage APIs, controlled manager state
and test registry rules. It does not prove a running world's contents, complete GTNH locale,
native pixels/animation or production performance. Formal game capture belongs to the Export
Agent and must keep these scope/freshness limits visible.
