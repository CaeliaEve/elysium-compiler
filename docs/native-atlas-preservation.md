# Native atlas preservation

NESQL samples item and fluid animations against their native atlas. Each bounded
client slice must restore the pixels that existed at entry, including native
interpolation and mip levels, even if drawing throws.

Uncompressed color-renderable formats use an FBO read of the touched sprite
region. S3TC DXT1 RGB/RGBA, DXT3 and DXT5 cannot be FBO color attachments. For
those formats, the reader preserves the original encoded blocks and restores
them with `glCompressedTexSubImage2D`. Reading RGBA and recompressing it is not an
exact restoration and is deliberately avoided.

The GL 1.3 compressed read API reads a whole mip level. A staging buffer is
reused across sprites during snapshot construction, bounded at 64 MiB per mip;
only touched encoded blocks remain in each sprite patch. Unknown compression
formats, inconsistent dimensions/encoded sizes and over-budget reads fail
explicitly. S3TC frame uploads must be block-aligned, except a whole tiny mip;
unaligned sprites fail before any animation upload changes the atlas. This
boundary is not permission to use missing textures, approximate pixels or drop
native animation frames.

The offscreen real-OpenGL fixture covers all four admitted formats, allocated
mip chains, a proper subregion, intentional drawing failure and rejection of an
unaligned subregion. It compares every original compressed byte of the complete
test atlas after restoration. Existing independent item/fluid atlas and clock
restoration cases remain in the same targeted suite.

Historical job `dc65ae0b-dc6c-4047-b79f-17e24597784e` failed at indices 28, 156 and
207 without the expanded format evidence. Those failures are **not** established
to be S3TC-related by this fixture. They remain pending native confirmation;
new failures include actual sprite, mip, dimensions, internal format, compression
state and framebuffer status. No live game or export has been run for this fix.
