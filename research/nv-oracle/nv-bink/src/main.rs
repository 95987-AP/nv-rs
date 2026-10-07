//! nv-bink: decode a Bink movie with the game's own `binkw32.dll` and record
//! what it produced for every frame, so a Rust decoder can be checked against
//! the library the game really uses.
//!
//! ```text
//! nv-bink <binkw32.dll> <movie.bik> <out-dir> [options]
//!   --surface N     BinkCopyToBuffer surface type (default 15, YV12)
//!   --first N       first frame to record (default 0; earlier frames are
//!                   still decoded, Bink frames depend on the ones before)
//!   --count N       how many frames to record (default: to the end)
//!   --dump A,B,...  also write these frames' buffers to frame_NNNNN.raw
//!   --audio T       also decode audio track T of every frame with
//!                   BinkOpenTrack/BinkGetTrackData
//!   --precision P   x87 precision control while decoding: 24, 53 or 64
//!                   bits (default: leave the process's setting, which is
//!                   53 bits for a 32-bit MSVC program)
//! ```
//!
//! The library's audio code is x87 floating point, so its exact output
//! depends on the precision control of the thread that decodes. Direct3D 9
//! sets 24 bits unless a device is created with D3DCREATE_FPU_PRESERVE, so a
//! game's thread may differ from this program's default.
//!
//! Writes `frames.tsv` (frame number, then the SHA-256 of each plane of the
//! copied buffer in buffer order) and `header.txt` (what the library reports
//! for the movie). For YV12 the buffer is the luma plane (pitch = width), then
//! two chroma planes of half width and half height. The output is a private
//! recording of the user's own game files; never commit it.
//!
//! Sound is switched off before the movie is opened (`BinkSetSoundTrack(0,
//! NULL)`), so no sound device is touched. With `--audio` the track's decoded
//! samples are taken from the library's raw track interface instead
//! (`BinkGetTrackData` once per frame, after `BinkDoFrame`), and written to
//! `audio.pcm` (as the library gives them: interleaved, 16-bit little-endian
//! for a 16-bit track) and `audio.tsv` (frame number, byte count, SHA-256 of
//! that frame's bytes). `header.txt` then also gives the track's rate, bits,
//! channels and largest size as the library reports them.

#[cfg(all(windows, target_arch = "x86"))]
mod run {
    use nv_oracle_core::sha256;
    use std::ffi::c_void;
    use std::io::Write;

    type BinkOpen = unsafe extern "system" fn(*const u8, u32) -> *mut u32;
    type BinkSetSoundTrack = unsafe extern "system" fn(u32, *const u32);
    type BinkDoFrame = unsafe extern "system" fn(*mut u32) -> i32;
    type BinkCopyToBuffer =
        unsafe extern "system" fn(*mut u32, *mut c_void, i32, u32, u32, u32, u32) -> i32;
    type BinkNextFrame = unsafe extern "system" fn(*mut u32);
    type BinkClose = unsafe extern "system" fn(*mut u32);
    type BinkGetError = unsafe extern "system" fn() -> *const u8;
    type BinkOpenTrack = unsafe extern "system" fn(*mut u32, u32) -> *mut u32;
    type BinkGetTrackData = unsafe extern "system" fn(*mut u32, *mut u8) -> u32;
    type BinkCloseTrack = unsafe extern "system" fn(*mut u32);

    /// BINKCOPYALL: copy the whole frame, not only the parts that changed.
    const COPY_ALL: u32 = 0x8000_0000;

    struct Options {
        dll: String,
        movie: String,
        out: std::path::PathBuf,
        surface: u32,
        first: u32,
        count: Option<u32>,
        dump: Vec<u32>,
        audio: Option<u32>,
        precision: Option<u32>,
    }

    fn parse() -> Result<Options, String> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        if args.len() < 3 {
            return Err("usage: nv-bink <binkw32.dll> <movie.bik> <out-dir> [--surface N] [--first N] [--count N] [--dump A,B] [--audio T] [--precision 24|53|64]".into());
        }
        let mut o = Options {
            dll: args[0].clone(),
            movie: args[1].clone(),
            out: args[2].clone().into(),
            surface: 15,
            first: 0,
            count: None,
            dump: Vec::new(),
            audio: None,
            precision: None,
        };
        let mut i = 3;
        while i < args.len() {
            let v = args
                .get(i + 1)
                .ok_or(format!("{} needs a value", args[i]))?;
            let num = |s: &str| s.parse::<u32>().map_err(|e| format!("{s}: {e}"));
            match args[i].as_str() {
                "--surface" => o.surface = num(v)?,
                "--first" => o.first = num(v)?,
                "--count" => o.count = Some(num(v)?),
                "--audio" => o.audio = Some(num(v)?),
                "--precision" => o.precision = Some(num(v)?),
                "--dump" => {
                    for p in v.split(',') {
                        o.dump.push(num(p)?);
                    }
                }
                other => return Err(format!("unknown option {other}")),
            }
            i += 2;
        }
        Ok(o)
    }

    unsafe fn proc<T: Copy>(module: nv_win::Handle, name: &str) -> Result<T, String> {
        let z = format!("{name}\0");
        let p = nv_win::GetProcAddress(module, z.as_ptr());
        if p.is_null() {
            return Err(format!("binkw32.dll has no export {name}"));
        }
        Ok(std::mem::transmute_copy(&p))
    }

    /// The planes of a copied buffer, as (offset, length) pairs.
    fn planes(surface: u32, w: usize, h: usize) -> Vec<(usize, usize)> {
        match surface {
            15 => {
                let y = w * h;
                let c = (w / 2) * (h / 2);
                vec![(0, y), (y, c), (y + c, c)]
            }
            _ => vec![(0, buffer_len(surface, w, h))],
        }
    }

    fn pitch(surface: u32, w: usize) -> usize {
        match surface {
            1 | 2 => w * 3,
            3..=6 => w * 4,
            7..=14 => w * 2,
            15 => w,
            _ => w,
        }
    }

    fn buffer_len(surface: u32, w: usize, h: usize) -> usize {
        match surface {
            15 => w * h + 2 * (w / 2) * (h / 2),
            _ => pitch(surface, w) * h,
        }
    }

    fn fpu_control_word() -> u16 {
        let mut cw: u16 = 0;
        unsafe {
            core::arch::asm!("fnstcw word ptr [{0}]", in(reg) &mut cw, options(nostack));
        }
        cw
    }

    fn set_fpu_control_word(cw: u16) {
        unsafe {
            core::arch::asm!("fldcw word ptr [{0}]", in(reg) &cw, options(nostack));
        }
    }

    pub fn main() -> Result<(), String> {
        let o = parse()?;
        if let Some(p) = o.precision {
            let pc = match p {
                24 => 0u16,
                53 => 2,
                64 => 3,
                other => return Err(format!("--precision {other}: use 24, 53 or 64")),
            };
            set_fpu_control_word((fpu_control_word() & !0x0300) | (pc << 8));
        }
        std::fs::create_dir_all(&o.out).map_err(|e| format!("{}: {e}", o.out.display()))?;
        unsafe {
            let dll = format!("{}\0", o.dll);
            let module = nv_win::LoadLibraryA(dll.as_ptr());
            if module.is_null() {
                return Err(format!(
                    "LoadLibrary {} failed ({})",
                    o.dll,
                    nv_win::GetLastError()
                ));
            }
            let open: BinkOpen = proc(module, "_BinkOpen@8")?;
            let set_tracks: BinkSetSoundTrack = proc(module, "_BinkSetSoundTrack@8")?;
            let do_frame: BinkDoFrame = proc(module, "_BinkDoFrame@4")?;
            let copy: BinkCopyToBuffer = proc(module, "_BinkCopyToBuffer@28")?;
            let next: BinkNextFrame = proc(module, "_BinkNextFrame@4")?;
            let close: BinkClose = proc(module, "_BinkClose@4")?;
            let get_error: BinkGetError = proc(module, "_BinkGetError@0")?;
            let open_track: BinkOpenTrack = proc(module, "_BinkOpenTrack@8")?;
            let track_data: BinkGetTrackData = proc(module, "_BinkGetTrackData@8")?;
            let close_track: BinkCloseTrack = proc(module, "_BinkCloseTrack@4")?;

            set_tracks(0, std::ptr::null());
            let movie = format!("{}\0", o.movie);
            let bink = open(movie.as_ptr(), 0);
            if bink.is_null() {
                let e = get_error();
                let msg = if e.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(e as *const i8)
                        .to_string_lossy()
                        .into_owned()
                };
                return Err(format!("BinkOpen failed: {msg}"));
            }
            // The HBINK structure starts Width, Height, Frames, FrameNum.
            let w = *bink as usize;
            let h = *bink.add(1) as usize;
            let frames = *bink.add(2);
            let mut header =
                std::fs::File::create(o.out.join("header.txt")).map_err(|e| e.to_string())?;
            let dll_hash =
                sha256::digest_reader(std::fs::File::open(&o.dll).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            writeln!(
                header,
                "dll\t{}\ndll_sha256\t{}\nmovie\t{}\nwidth\t{w}\nheight\t{h}\nframes\t{frames}\nsurface\t{}\nfpu_control_word\t{:04x}",
                o.dll,
                nv_oracle_core::hex::encode(&dll_hash),
                o.movie,
                o.surface,
                fpu_control_word()
            )
            .map_err(|e| e.to_string())?;

            // HBINKTRACK starts Frequency, Bits, Channels, MaxSize.
            let track = match o.audio {
                Some(t) => {
                    let tr = open_track(bink, t);
                    if tr.is_null() {
                        return Err(format!("BinkOpenTrack({t}) failed"));
                    }
                    writeln!(
                        header,
                        "track\t{t}\ntrack_rate\t{}\ntrack_bits\t{}\ntrack_channels\t{}\ntrack_max_size\t{}",
                        *tr,
                        *tr.add(1),
                        *tr.add(2),
                        *tr.add(3)
                    )
                    .map_err(|e| e.to_string())?;
                    Some(tr)
                }
                None => None,
            };
            let mut pcm_buf = vec![0u8; 1 << 20];
            let mut pcm = match track {
                Some(_) => Some(
                    std::fs::File::create(o.out.join("audio.pcm")).map_err(|e| e.to_string())?,
                ),
                None => None,
            };
            let mut audio_tsv = match track {
                Some(_) => Some(
                    std::fs::File::create(o.out.join("audio.tsv")).map_err(|e| e.to_string())?,
                ),
                None => None,
            };

            let last = match o.count {
                Some(c) => (o.first + c).min(frames),
                None => frames,
            };
            let mut buf = vec![0u8; buffer_len(o.surface, w, h)];
            let mut tsv =
                std::fs::File::create(o.out.join("frames.tsv")).map_err(|e| e.to_string())?;
            for f in 0..last {
                do_frame(bink);
                if let Some(tr) = track {
                    let n = track_data(tr, pcm_buf.as_mut_ptr()) as usize;
                    if n > pcm_buf.len() {
                        return Err(format!("frame {f}: {n} audio bytes overran the buffer"));
                    }
                    let bytes = &pcm_buf[..n];
                    if let Some(file) = pcm.as_mut() {
                        file.write_all(bytes).map_err(|e| e.to_string())?;
                    }
                    if let Some(file) = audio_tsv.as_mut() {
                        writeln!(file, "{f}\t{n}\t{}", sha256::hex_digest(bytes))
                            .map_err(|e| e.to_string())?;
                    }
                }
                if f >= o.first {
                    let r = copy(
                        bink,
                        buf.as_mut_ptr() as *mut c_void,
                        pitch(o.surface, w) as i32,
                        h as u32,
                        0,
                        0,
                        o.surface | COPY_ALL,
                    );
                    let mut line = format!("{f}\t{r}");
                    for (start, len) in planes(o.surface, w, h) {
                        line.push('\t');
                        line.push_str(&sha256::hex_digest(&buf[start..start + len]));
                    }
                    writeln!(tsv, "{line}").map_err(|e| e.to_string())?;
                    if o.dump.contains(&f) {
                        std::fs::write(o.out.join(format!("frame_{f:05}.raw")), &buf)
                            .map_err(|e| e.to_string())?;
                    }
                }
                if f + 1 < last {
                    next(bink);
                }
            }
            if let Some(tr) = track {
                close_track(tr);
            }
            close(bink);
            let cw = fpu_control_word();
            if let Some(p) = o.precision {
                let want = match p {
                    24 => 0,
                    53 => 2,
                    _ => 3,
                };
                if (cw >> 8) & 3 != want {
                    return Err(format!(
                        "the library changed the x87 control word to {cw:04x}"
                    ));
                }
            }
            println!(
                "{w}x{h}, {frames} frames; recorded {} to {}",
                last.saturating_sub(o.first),
                o.out.display()
            );
        }
        Ok(())
    }
}

#[cfg(all(windows, target_arch = "x86"))]
fn main() {
    if let Err(e) = run::main() {
        eprintln!("nv-bink: {e}");
        std::process::exit(1);
    }
}

#[cfg(not(all(windows, target_arch = "x86")))]
fn main() {
    eprintln!("nv-bink runs only as a 32-bit Windows program (the game's binkw32.dll is 32-bit)");
    std::process::exit(1);
}
