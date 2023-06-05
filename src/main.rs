use std::fs;
use std::io::prelude::*;
use std::path::Path;
#[derive(Debug)]
struct MBoxList {
    magic: u16,
    padding: u16,
    version: u32,
    number_of_boxes: u32,
    box_names: Vec<String>,
}

#[derive(Debug)]
struct MBoxInfo {
    magic: u16,
    padding: u16,
    title_id: u32,
    private_id: u32,
    flag: u8,
    is_active: u8,
    padding2: u16,
    hmac_key: [u8; 32],
    zero: u32,
    timestamp_last_accessed: [u8; 12],
    flag1: u8,
    flag2: u8,
    flag3: u8,
    flag4: u8,
    timestamp_last_received: [u8; 12],
    reserved: [u8; 16],
}
// Add these structs to represent the remaining file data structures
#[derive(Debug)]
struct MBoxData {
    icon: Vec<u8>,
    game_title: String,
    title_id: u64,
}

#[derive(Debug)]
struct BoxInfo {
    magic: u16,
    padding: u16,
    file_size: u32,
    max_box_size: u32,
    current_box_size: u32,
    max_message_count: u32,
    current_message_count: u32,
    max_batch_size: u32,
    max_message_size: u32,
}

#[derive(Debug)]
struct MessageFile {
    magic: u16,
    padding: u16,
    message_size: u32,
    header_extra_headers_size: u32,
    body_size: u32,
    title_id: u32,
    title_id_reserve: u32,
    group_id: u32,
    unknown_id: u32,
    message_id: u64,
    message_version: u32,
    message_id2: u64,
    flags: u8,
    send_method: u8,
    is_unopen: u8,
    is_new: u8,
    sender_id: u64,
    sender_id2: u64,
    timestamp_sent: [u8; 12],
    timestamp_received: [u8; 12],
    timestamp_created: [u8; 12],
    send_count: u8,
    propagation_count: u8,
    tag: u16,
}

// Add these parsing functions for the remaining file types
fn parse_mbox_data(icon_data: &[u8], game_title_data: &[u8], title_id_data: &[u8]) -> MBoxData {
    let icon = icon_data.to_vec();
    let game_title = String::from_utf16_lossy(
        &game_title_data[..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<u16>>(),
    );
    let title_id = u64::from_le_bytes([
        title_id_data[0],
        title_id_data[1],
        title_id_data[2],
        title_id_data[3],
        title_id_data[4],
        title_id_data[5],
        title_id_data[6],
        title_id_data[7],
    ]);

    MBoxData {
        icon,
        game_title,
        title_id,
    }
}

fn parse_box_info(data: &[u8]) -> BoxInfo {
    let magic = u16::from_le_bytes([data[0], data[1]]);
    let padding = u16::from_le_bytes([data[2], data[3]]);
    let file_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let max_box_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let current_box_size = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
    let max_message_count = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
    let current_message_count = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
    let max_batch_size = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
    let max_message_size = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);

    BoxInfo {
        magic,
        padding,
        file_size,
        max_box_size,
        current_box_size,
        max_message_count,
        current_message_count,
        max_batch_size,
        max_message_size,
    }
}
#[derive(Debug)]
struct SaveFile {
    cec: CEC,
}

#[derive(Debug)]
struct CEC {
    m_box_list: MBoxList,
    titles: Vec<Title>,
}

#[derive(Debug)]
struct Title {
    m_box_info: MBoxInfo,
    m_box_data: MBoxData,
    inbox: Box,
    outbox: Box,
}

#[derive(Debug)]
struct Box {
    info: BoxInfo,
    messages: Vec<MessageFile>,
}
fn create_save_file(m_box_list: MBoxList, titles: Vec<Title>) -> SaveFile {
    SaveFile {
        cec: CEC { m_box_list, titles },
    }
}

fn main() {
    let mount_path = Path::new("./mount");

    // Read and parse MBoxList____
    let mbox_list_path = mount_path.join("CEC/MBoxList____");
    let mbox_list_data = fs::read(mbox_list_path).expect("Error reading MBoxList____");
    let m_box_list = parse_mbox_list(&mbox_list_data);
    let mut titles = Vec::new();

    for box_name in &m_box_list.box_names {
        let box_path = mount_path.join("CEC").join(box_name);

        let mbox_data_icon_path = box_path.join("MBoxData.001");
        let mbox_data_game_title_path = box_path.join("MBoxData.010");
        let mbox_data_title_id_path = box_path.join("MBoxData.050");

        let mbox_data_icon = fs::read(mbox_data_icon_path).expect("Error reading MBoxData.001");
        let mbox_data_game_title =
            fs::read(mbox_data_game_title_path).expect("Error reading MBoxData.010");
        let mbox_data_title_id =
            fs::read(mbox_data_title_id_path).expect("Error reading MBoxData.050");

        let mbox_data =
            parse_mbox_data(&mbox_data_icon, &mbox_data_game_title, &mbox_data_title_id);

        let inbox_path = box_path.join("InBox___");
        let outbox_path = box_path.join("OutBox__");
        let folders = vec![inbox_path, outbox_path.clone()];
        let mut inbox_messages = Vec::new();
        let mut outbox_messages = Vec::new();

        for (i, folder) in folders.iter().enumerate() {
            if Path::new(folder).exists() {
                let files: Vec<_> = fs::read_dir(folder)
                    .expect("Error reading directory")
                    .filter_map(Result::ok)
                    .filter(|entry| entry.path().is_file())
                    .filter(|entry| {
                        entry
                            .path()
                            .file_name()
                            .map_or(false, |fname| fname.to_string_lossy().starts_with('_'))
                    })
                    .collect();

                for file in files {
                    let file_data = fs::read(file.path()).expect("Error reading file");
                    let message_file = parse_message_file(&file_data);
                    if i == 0 {
                        inbox_messages.push(message_file);
                    } else {
                        outbox_messages.push(message_file);
                    }
                }
            } else {
                println!("{:?} folder does not exist.", folder);
            }
        }
        let box_info_path = box_path.join("InBox___").join("BoxInfo_____");
        let box_info_data = fs::read(box_info_path).expect("Error reading BoxInfo_____");

        let outbox_info_path = outbox_path.join("BoxInfo_____");
        let outbox_info_data = fs::read(outbox_info_path).expect("Error reading BoxInfo_____");

        let mbox_info_data =
            fs::read(box_path.join("MBoxInfo____")).expect("Error reading MBoxInfo____");

        let inbox = Box {
            info: parse_box_info(&box_info_data),
            messages: inbox_messages,
        };

        let outbox = Box {
            info: parse_box_info(&outbox_info_data),
            messages: outbox_messages,
        };

        let title = Title {
            m_box_info: parse_mbox_info(&mbox_info_data),
            m_box_data: mbox_data,
            inbox,
            outbox,
        };

        titles.push(title);
    }

    let save_file = create_save_file(m_box_list, titles);
    // println!("{:?}", save_file);
}

fn parse_message_file(data: &[u8]) -> MessageFile {
    use std::convert::TryInto;

    let magic = u16::from_le_bytes(data[0..2].try_into().unwrap());
    let padding = u16::from_le_bytes(data[2..4].try_into().unwrap());
    let message_size = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let header_extra_headers_size = u32::from_le_bytes(data[8..12].try_into().unwrap());
    let body_size = u32::from_le_bytes(data[12..16].try_into().unwrap());

    MessageFile {
        magic,
        padding,
        message_size,
        header_extra_headers_size,
        body_size,
        title_id: u32::from_le_bytes(data[16..20].try_into().unwrap()),
        title_id_reserve: u32::from_le_bytes(data[20..24].try_into().unwrap()),
        group_id: u32::from_le_bytes(data[24..28].try_into().unwrap()),
        unknown_id: u32::from_le_bytes(data[28..32].try_into().unwrap()),
        message_id: u64::from_le_bytes(data[32..40].try_into().unwrap()),
        message_version: u32::from_le_bytes(data[40..44].try_into().unwrap()),
        message_id2: u64::from_le_bytes(data[44..52].try_into().unwrap()),
        flags: data[52],
        send_method: data[53],
        is_unopen: data[54],
        is_new: data[55],
        sender_id: u64::from_le_bytes(data[56..64].try_into().unwrap()),
        sender_id2: u64::from_le_bytes(data[64..72].try_into().unwrap()),
        timestamp_sent: data[72..84].try_into().unwrap(),
        timestamp_received: data[84..96].try_into().unwrap(),
        timestamp_created: data[96..108].try_into().unwrap(),
        send_count: data[108],
        propagation_count: data[109],
        tag: u16::from_le_bytes(data[110..112].try_into().unwrap()),
    }
}

fn parse_mbox_list(data: &[u8]) -> MBoxList {
    let magic = u16::from_le_bytes([data[0], data[1]]);
    let padding = u16::from_le_bytes([data[2], data[3]]);
    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let number_of_boxes = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);

    let mut box_names = Vec::new();
    for i in 0..number_of_boxes as usize {
        let box_name_start = 0x0C + i * 16;
        let box_name_end = box_name_start + 16;
        let box_name_bytes = &data[box_name_start..box_name_end];
        let box_name = String::from_utf8_lossy(box_name_bytes)
            .trim_end_matches('\0')
            .to_string();
        box_names.push(box_name);
    }

    MBoxList {
        magic,
        padding,
        version,
        number_of_boxes,
        box_names,
    }
}

fn parse_mbox_info(data: &[u8]) -> MBoxInfo {
    let magic = u16::from_le_bytes([data[0], data[1]]);
    let padding = u16::from_le_bytes([data[2], data[3]]);
    let title_id = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let private_id = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let flag = data[12];
    let is_active = data[13];
    let padding2 = u16::from_le_bytes([data[14], data[15]]);

    let mut hmac_key = [0; 32];
    hmac_key.copy_from_slice(&data[16..48]);

    let zero = u32::from_le_bytes([data[48], data[49], data[50], data[51]]);

    let mut timestamp_last_accessed = [0; 12];
    timestamp_last_accessed.copy_from_slice(&data[52..64]);

    let flag1 = data[64];
    let flag2 = data[65];
    let flag3 = data[66];
    let flag4 = data[67];

    let mut timestamp_last_received = [0; 12];
    timestamp_last_received.copy_from_slice(&data[68..80]);

    let mut reserved = [0; 16];
    reserved.copy_from_slice(&data[80..96]);

    MBoxInfo {
        magic,
        padding,
        title_id,
        private_id,
        flag,
        is_active,
        padding2,
        hmac_key,
        zero,
        timestamp_last_accessed,
        flag1,
        flag2,
        flag3,
        flag4,
        timestamp_last_received,
        reserved,
    }
}
