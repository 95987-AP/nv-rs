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

## Audio decoder: `bink::AudioDecoder`

Bink audio, DCT variant. From the DLL (read in Ghidra):

- Raw track interface: `BinkGetTrackData` (0x18012eb0) walks the frame's
  audio packets, takes the first u32 of the track's packet as the number of
  PCM bytes to produce, and runs the block decoder (0x18018800) until it
  has them; the last block's output may be cut short.
- Set-up (0x180185d0): frame length 512, 1024 or 2048 below 22050 Hz,
  below 44100 Hz, and otherwise. Band edges in units of two coefficients:
  `max(1, f * (N / 2) / ((rate + 1) / 2))` for the band frequencies at
  0x1805e710 (0, 100, 200 ... 15500 Hz; the list starts at 0) up to the first
  that is not below half the rate, then `N / 2`. Scale `2 / sqrt(N)`.
- Block (0x18018210): two unused bits; per channel two 29-bit leading
  coefficients (5-bit exponent, 23-bit mantissa, sign), each the mantissa
  times the float at 0x1805eb78 + 4 * exponent (2^(e - 23); entries 24 and up
  are not powers of two, see `audio.rs`); one 8-bit level code per band,
  looked up in the 256 floats at 0x1805e778 (1.1652^code, not clamped); then
  the coefficients (0x18017f20): a run flag (8 coefficients, or 8 times the
  entry of the run table at 0x1805e700 picked by 4 bits), a 4-bit width,
  and either zeros or width-bit magnitudes with a sign bit, times the
  current band's level. The block ends at the next 32-bit word.
- Each channel goes through an inverse DCT with weight 1 on coefficient 0
  (Takuya Ooura's `ddct(n, 1, ...)`, single-precision x87 code).
- Output (0x18017e40): sample times the scale, rounded by `FISTP` (to
  nearest even), saturated to 16 bits, interleaved.
- Overlap (0x18018800): the first N/16 samples per channel of each block
  but the first are crossfaded with the previous block's last N/16 in
  integers, `(prev * (n - i) + cur * i)` divided by `n` as an unsigned
  number (n counts interleaved samples), and each block outputs its first
  `N - N/16` samples per channel.

`crates/bink` follows all of this except the transform, which it computes in
double precision through a complex FFT; the DLL's single-precision x87
transform gives results that depend on the thread's precision control.

### Checked against the game's library

`nv-bink ... --audio 0 --precision 24|53` records the DLL's output through
`BinkGetTrackData` with the x87 precision control set to 24 or 53 bits (the
DLL left it unchanged). `nvinspect FNVIntro.bik audio OUT` writes the Rust
decoder's.

| Compared | Samples | Identical | Differ by 1 | Larger |
| --- | --- | --- | --- | --- |
| DLL at 24 bits vs DLL at 53 bits | 27,814,372 | 99.9799% | 5,588 | 0 |
| `crates/bink` vs DLL at 24 bits | 27,814,372 | 99.9825% | 4,863 | 0 |
| `crates/bink` vs DLL at 53 bits | 27,814,372 | 99.9847% | 4,269 | 0 |

Same length (289.733 s at 48 kHz, stereo) and never more than one step
apart, which is as close as the library is to itself under the two
precision settings. Which setting the game's decoding thread has is not
established yet (Direct3D 9 lowers it to 24 bits unless the device is created
with `D3DCREATE_FPU_PRESERVE`, and Bink may decode on its own thread).
Decoding the whole track takes 0.6 s.

## Not done yet

- How the game shows the movie: the `PlayBink` arguments (`1 1 0 1`), the
  YUV-to-RGB conversion and scaling, whether and how it can be skipped,
  sound volume, and what the game does when it ends. These need the
  `PlayBink` handler in `FalloutNV.exe`.
- The viewer does not play movies yet; `--new-game` still starts after it.
