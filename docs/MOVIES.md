# Movies (Bink): M1 working evidence

The opening quest starts the game with a movie. `VCG00` stage 0 runs
`PlayBink "FNVIntro.bik" 1 1 0 1`, then `SetStage VCG00 90` (the source
marks this "TEMP: DEMO CODE"; the burial scene of stages 17 onwards, with its
`TriggerScreenSplatter` calls, is never reached in the shipped game). Seen
with `nvinspect <Data> source VCG00` on the official data. The base game's
only movie is `Data\Video\FNVIntro.bik`.

## The file

`nvinspect Data\Video\FNVIntro.bik info` on the 1.4.0.525 install:

- Bink revision `i`, 1280 x 720, 8,692 frames at 10000000/333333 frames per
  second (30.00003), 289.7 s, 336.9 MiB.
- Video flags 0 (no alpha plane, not grey). One keyframe (frame 0).
- One audio track: 48000 Hz, two channels, the DCT variant, 16-bit, 8.8 MiB.

## Video decoder: `crates/bink`

A pure Rust Bink 1 decoder (no external crates, like `crates/mp3`).
`bink::Movie` reads the header, track list and frame index;
`bink::Decoder` decodes the video data of each frame in order into Y, U and
V planes.

Its fixed tables were read from the install's own `binkw32.dll`
(SHA-256 `c6c06e2d21f2179f0b7fbe829a4b17a7b371440d2c679253ec86b64d421185bd`,
image base 0x18000000), with the address of each in
`crates/bink/src/tables.rs`:

| Table | Address | How it was confirmed |
| --- | --- | --- |
| Coefficient scan order | 0x1804ca20 | A permutation of the 64 block positions (test). |
| 16 run-fill patterns | 0x1804ca68 | Read by the plane decoder at 0x1801aac0; each a permutation (test). |
| Huffman code lengths and codes | 0x1804ce70, 0x1804cf78 | All 16 trees are complete prefix codes when read from the low bit (test). The codes follow 8 bytes of padding. |
| Block-type run lengths 4, 8, 12, 32 | 0x180487c8 | Indexed from 0x180487bc (symbol minus 12) by the block-type reader at 0x18019860. |
| Intra and inter dequantization, 16 x 64 each | 0x180489a0, 0x180499c0 | Indexed by the inverse DCTs at 0x1801cb30/0x1801ce60 (intra) and 0x1801d230 (inter). |

From the DLL's inverse DCT (0x1801cb30, and the 16x16 and add variants at
0x1801ce60 and 0x1801d230), read in Ghidra:

- Each coefficient (16-bit) is multiplied by its table entry in natural
  row-major order and shifted right by 11 as the columns are read, with
  32-bit wrap-around. A column whose entries below the first are all zero
  is filled with the first.
- The 8-point transform uses 2896, 3784, -5352 and 2217 with `>> 11`.
- Each output is `(x + 127) >> 8` kept to its low 8 bits: no clamping.
  Inter blocks add that byte to the predicted pixel, also without clamping.
- The frame function (0x1801c990) decodes the alpha plane only with video
  flag 0x100000 and the chroma planes only without 0x20000.

The rest of the bitstream (bundles, their Huffman trees, block types,
coefficient and residue coding, plane order and 32-bit plane alignment)
follows the public descriptions of the format.

### Checked against the game's library

`research/nv-oracle`'s `nv-bink` decodes the movie with the install's
`binkw32.dll` and records the SHA-256 of each frame's Y, V and U planes
(`BinkCopyToBuffer`, YV12, `BINKCOPYALL`). `nvinspect FNVIntro.bik frames OUT`
writes the same from `crates/bink`.

On 2026-10-06, on the maintainer's machine, **all 8,692 frames of
`FNVIntro.bik` were identical in all three planes**, with no decode errors.
The DLL took about 44 s for the whole movie and `crates/bink` (release
build) 39.5 s.

Both hash files are private recordings and are not committed. Rendering a
dumped frame's planes as Y, then Cr, then Cb gave the expected colours, which
confirmed the plane order of the library's YV12 output.

Only revision `i` was compared. The decoder also accepts revisions `b` to
`k` (revision `k`'s whole-plane fill and the pre-`i` colour coding are
handled as publicly described) but those paths have no evidence here.

## Not done yet

- Audio (Bink audio, DCT variant, 48 kHz stereo) is not decoded.
- How the game shows the movie: the `PlayBink` arguments (`1 1 0 1`), the
  YUV-to-RGB conversion and scaling, whether and how it can be skipped,
  sound volume, and what the game does when it ends. These need the
  `PlayBink` handler in `FalloutNV.exe`.
- The viewer does not play movies yet; `--new-game` still starts after it.
