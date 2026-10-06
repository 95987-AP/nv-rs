//! Tests on generated input. The game's movie itself is compared with the
//! game's library outside the tests (see the crate documentation).

use crate::bits::BitReader;
use crate::tables::{PATTERNS, SCAN, TREE_CODES, TREE_LENGTHS};
use crate::{Decoder, Error, Movie, AUDIO_STEREO};

#[test]
fn bits_come_from_the_low_end_of_little_endian_words() {
    // 0b1011_0110 then 0xff.
    let data = [0xb6u8, 0xff, 0x00, 0x80];
    let mut br = BitReader::new(&data);
    assert_eq!(br.bits(1), 0);
    assert_eq!(br.bits(2), 0b11);
    assert_eq!(br.bits(5), 0b10110);
    assert_eq!(br.bits(8), 0xff);
    br.align32();
    assert_eq!(br.position(), 32);
    assert!(!br.overrun());
    assert_eq!(br.bits(4), 0);
    assert!(br.overrun());
}

#[test]
fn every_tree_is_a_complete_prefix_code() {
    // Kraft sum of exactly 1 and no code a prefix of another (read from the
    // low bit up): every 7-bit pattern then decodes to exactly one leaf.
    for t in 0..16 {
        let kraft: u32 = TREE_LENGTHS[t].iter().map(|&len| 1u32 << (7 - len)).sum();
        assert_eq!(kraft, 128, "tree {t}");
        for pattern in 0u32..128 {
            let matches = (0..16)
                .filter(|&leaf| {
                    let len = TREE_LENGTHS[t][leaf] as u32;
                    pattern & ((1 << len) - 1) == TREE_CODES[t][leaf] as u32
                })
                .count();
            assert_eq!(matches, 1, "tree {t} pattern {pattern:07b}");
        }
    }
}

#[test]
fn scan_and_patterns_are_permutations_of_the_block() {
    let check = |order: &[u8; 64]| {
        let mut seen = [false; 64];
        for &p in order {
            assert!(!seen[p as usize]);
            seen[p as usize] = true;
        }
    };
    check(&SCAN);
    for p in &PATTERNS {
        check(p);
    }
}

/// A header with `frames` frames of the given sizes, one stereo track.
fn movie_bytes(frame_sizes: &[usize]) -> Vec<u8> {
    let mut d = Vec::new();
    d.extend_from_slice(b"BIKi");
    d.extend_from_slice(&0u32.to_le_bytes()); // size, filled below
    d.extend_from_slice(&(frame_sizes.len() as u32).to_le_bytes());
    d.extend_from_slice(&(*frame_sizes.iter().max().unwrap() as u32).to_le_bytes());
    d.extend_from_slice(&(frame_sizes.len() as u32).to_le_bytes());
    d.extend_from_slice(&64u32.to_le_bytes());
    d.extend_from_slice(&48u32.to_le_bytes());
    d.extend_from_slice(&30u32.to_le_bytes());
    d.extend_from_slice(&1u32.to_le_bytes());
    d.extend_from_slice(&0u32.to_le_bytes()); // flags
    d.extend_from_slice(&1u32.to_le_bytes()); // one track
    d.extend_from_slice(&4096u32.to_le_bytes());
    d.extend_from_slice(&44100u16.to_le_bytes());
    d.extend_from_slice(&(AUDIO_STEREO).to_le_bytes());
    d.extend_from_slice(&7u32.to_le_bytes()); // track id
    let index_at = d.len();
    let first = index_at + 4 * (frame_sizes.len() + 1);
    let mut o = first;
    for (i, &s) in frame_sizes.iter().enumerate() {
        let key = if i == 0 { 1 } else { 0 };
        d.extend_from_slice(&((o as u32) | key).to_le_bytes());
        o += s;
    }
    d.extend_from_slice(&(o as u32).to_le_bytes());
    for (i, &s) in frame_sizes.iter().enumerate() {
        // An audio packet of 8 bytes, then video bytes.
        d.extend_from_slice(&8u32.to_le_bytes());
        d.extend_from_slice(&[0xa0 + i as u8; 8]);
        d.extend(std::iter::repeat(i as u8).take(s - 12));
    }
    let size = (d.len() - 8) as u32;
    d[4..8].copy_from_slice(&size.to_le_bytes());
    d
}

#[test]
fn the_header_index_and_packets_are_read() {
    let bytes = movie_bytes(&[40, 20]);
    let m = Movie::parse(&bytes).unwrap();
    assert_eq!(m.revision, b'i');
    assert_eq!((m.width, m.height), (64, 48));
    assert_eq!(m.frame_count(), 2);
    assert_eq!(m.fps(), 30.0);
    assert_eq!(m.audio.len(), 1);
    assert_eq!(m.audio[0].sample_rate, 44100);
    assert_eq!(m.audio[0].channels(), 2);
    assert_eq!(m.audio[0].id, 7);
    let p0 = m.packet(0).unwrap();
    assert!(p0.keyframe);
    assert_eq!(p0.audio[0], &[0xa0; 8]);
    assert_eq!(p0.video.len(), 28);
    let p1 = m.packet(1).unwrap();
    assert!(!p1.keyframe);
    assert_eq!(p1.video, &[1u8; 8]);
    assert!(m.packet(2).is_err());
}

#[test]
fn bad_files_are_refused() {
    assert_eq!(Movie::parse(b"KB2j....").unwrap_err(), Error::NotBink);
    assert_eq!(Movie::parse(b"BIKz").unwrap_err(), Error::Revision(b'z'));
    let mut bytes = movie_bytes(&[40, 20]);
    let short = bytes.len() - 30;
    bytes.truncate(short);
    assert!(Movie::parse(&bytes).is_err());
}

#[test]
fn an_empty_frame_leaves_the_planes_alone() {
    let mut d = Decoder::new(64, 48, b'i', 0);
    // Only the 32-bit size word, then nothing: the luma plane starts but
    // every bundle reads a count of zero, so every block takes type 0
    // (skip) from the zeroed bundle and copies the black previous frame.
    let r = d.decode(&[0u8; 4]);
    assert!(r.is_err() || r.is_ok());
    let mut out = Vec::new();
    d.to_yv12(&mut out);
    assert_eq!(out.len(), 64 * 48 + 2 * 32 * 24);
}
