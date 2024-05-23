pub struct MsaHeader {
    pub sectors_per_track: u16,
    pub sides: u16,
    pub start_track: u16,
    pub end_track: u16,
}

pub fn read_msa_header(msa_data: &[u8]) -> Option<MsaHeader> {
    if msa_data.len() < 10 {
        return None;
    }

    let id_marker = u16::from_be_bytes([msa_data[0], msa_data[1]]);
    if id_marker != 0x0E0F {
        return None;
    }

    Some(MsaHeader {
        sectors_per_track: u16::from_be_bytes([msa_data[2], msa_data[3]]),
        sides: u16::from_be_bytes([msa_data[4], msa_data[5]]) + 1,
        start_track: u16::from_be_bytes([msa_data[6], msa_data[7]]),
        end_track: u16::from_be_bytes([msa_data[8], msa_data[9]]),
    })
}

pub fn process_data_track(track_data_length: usize, msa_data: &[u8], msa_index: usize, sectors_per_track: u16) -> std::io::Result<(Vec<u8>, usize)> {
    let mut st_data = Vec::with_capacity(512 * sectors_per_track as usize);
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

pub fn decode_msa_to_st(msa_data: &[u8]) -> Option<Vec<u8>> {
    let msa_header = read_msa_header(msa_data)?;

    let mut msa_index = 10;
    let mut st_data = Vec::with_capacity(512 * msa_header.sectors_per_track as usize * msa_header.sides as usize * (msa_header.end_track - msa_header.start_track + 1) as usize);

    for _track in msa_header.start_track..=msa_header.end_track {
        for _side in 0..msa_header.sides {
            let track_data_length = u16::from_be_bytes([msa_data[msa_index], msa_data[msa_index + 1]]) as usize;
            msa_index += 2;
            let (mut track_data, new_index) = process_data_track(track_data_length, msa_data, msa_index, msa_header.sectors_per_track).ok()?;
            st_data.append(&mut track_data);
            msa_index = new_index;
        }
    }
    Some(st_data)
}
