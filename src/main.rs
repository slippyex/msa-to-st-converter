use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

struct MsaHeader {
    sectors_per_track: u16,
    sides: u16,
    start_track: u16,
    end_track: u16,
}

fn read_msa_header(msa_data: &[u8]) -> io::Result<MsaHeader> {
    if msa_data.len() < 10 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "MSA data is too short."));
    }

    let id_marker = u16::from_be_bytes([msa_data[0], msa_data[1]]);
    if id_marker != 0x0E0F {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid MSA file. ID Marker does not match."));
    }

    Ok(MsaHeader {
        sectors_per_track: u16::from_be_bytes([msa_data[2], msa_data[3]]),
        sides: u16::from_be_bytes([msa_data[4], msa_data[5]]) + 1,
        start_track: u16::from_be_bytes([msa_data[6], msa_data[7]]),
        end_track: u16::from_be_bytes([msa_data[8], msa_data[9]]),
    })
}

fn process_data_track(track_data_length: usize, msa_data: &[u8], msa_index: usize, sectors_per_track: u16) -> io::Result<(Vec<u8>, usize)> {
    let mut st_data = Vec::new();
    let is_compressed = track_data_length < 512 * sectors_per_track as usize;
    let mut index = msa_index;
    let track_data_end = msa_index + track_data_length;

    while index < track_data_end {
        if !is_compressed {
            st_data.extend_from_slice(&msa_data[index..track_data_end]);
            index = track_data_end;
        } else {
            let byte = msa_data[index];
            index += 1;
            if byte != 0xE5 {
                st_data.push(byte);
            } else {
                let repeated_byte = msa_data[index];
                index += 1;
                let repeat_count = u16::from_be_bytes([msa_data[index], msa_data[index + 1]]) as usize;
                index += 2;
                st_data.extend(std::iter::repeat(repeated_byte).take(repeat_count));
            }
        }
    }
    Ok((st_data, index))
}

fn decode_msa_to_st(msa_data: &[u8]) -> io::Result<Vec<u8>> {
    let msa_header = read_msa_header(msa_data)?;

    let mut msa_index = 10;
    let mut st_data = Vec::new();
    for _track in msa_header.start_track..=msa_header.end_track {
        for _side in 0..msa_header.sides {
            let track_data_length = u16::from_be_bytes([msa_data[msa_index], msa_data[msa_index + 1]]) as usize;
            msa_index += 2;
            let (mut track_data, new_index) = process_data_track(track_data_length, msa_data, msa_index, msa_header.sectors_per_track)?;
            st_data.append(&mut track_data);
            msa_index = new_index;
        }
    }
    Ok(st_data)
}

fn traverse_directory(src_dir: &Path, dest_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dest_dir)?;
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest_dir.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            traverse_directory(&src_path, &dest_path)?;
        } else if let Some(ext) = src_path.extension().and_then(|s| s.to_str()) {
            if ext.to_lowercase() == "msa" {
                let original_basename = src_path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
                let st_path = PathBuf::from(dest_dir).join(format!("{}.st", original_basename));
                let mut msa_data = Vec::new();
                File::open(&src_path)?.read_to_end(&mut msa_data)?;
                let st_data = decode_msa_to_st(&msa_data)?;
                File::create(st_path)?.write_all(&st_data)?;
                println!("Converted {} successfully.", src_path.display());
            }
        }
    }

    // Folders cleanup. If after traversing files, the directory is empty, delete it.
    if fs::read_dir(dest_dir)?.next().is_none() {
        fs::remove_dir(dest_dir)?;
        println!("Directory {} is empty and was removed.", dest_dir.display());
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Please provide the source and destination directories.");
        std::process::exit(1);
    }
    let src_dir = Path::new(&args[1]);
    let dest_dir = Path::new(&args[2]);
    traverse_directory(src_dir, dest_dir)
}