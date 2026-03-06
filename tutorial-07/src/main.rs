fn main() {
    let mtu1: usize = 4020; // network 1 mtu (large)
    let mtu2: usize = 1500; // network 2 mtu (small, path mtu)
    let ip_header: usize = 20;
    let original_payload: usize = 4000;

    println!("ip packet fragmentation demo");
    println!("network 1 mtu: {} bytes", mtu1);
    println!("network 2 mtu: {} bytes (bottleneck)", mtu2);
    println!("ip header size: {} bytes", ip_header);
    println!("original payload: {} bytes", original_payload);
    println!("total datagram: {} bytes\n", original_payload + ip_header);

    fragment_packet(original_payload, mtu2, ip_header);
}

struct Fragment {
    fragment_number: usize,
    data_offset: usize,  // byte offset in original payload
    data_length: usize,  // bytes of payload in this fragment
    total_length: usize, // data + header
    offset_field: usize, // ip offset field (data_offset / 8)
    mf_flag: u8,         // more fragments flag
}

fn fragment_packet(payload_size: usize, path_mtu: usize, ip_header: usize) {
    let max_data_per_fragment = path_mtu - ip_header;

    // align max data to 8-byte boundary (required by ip fragmentation spec)
    let max_data_aligned = (max_data_per_fragment / 8) * 8;

    println!("max data per fragment (aligned to 8 bytes): {} bytes\n", max_data_aligned);

    let mut fragments: Vec<Fragment> = Vec::new();
    let mut bytes_sent: usize = 0;
    let mut fragment_number: usize = 1;

    while bytes_sent < payload_size {
        let remaining = payload_size - bytes_sent;

        // pick full chunk or whatever remains
        let data_length = if remaining > max_data_aligned {
            max_data_aligned
        } else {
            remaining
        };

        let is_last = bytes_sent + data_length >= payload_size;
        let mf_flag: u8 = if is_last { 0 } else { 1 };

        // offset field = bytes already sent / 8
        let offset_field = bytes_sent / 8;

        fragments.push(Fragment {
            fragment_number,
            data_offset: bytes_sent,
            data_length,
            total_length: data_length + ip_header,
            offset_field,
            mf_flag,
        });

        bytes_sent += data_length;
        fragment_number += 1;
    }

    // print table header
    println!(
        "{:<12} {:<16} {:<14} {:<14} {:<10}",
        "fragment #", "total length", "data length", "offset field", "mf flag"
    );
    println!("{}", "-".repeat(66));

    for f in &fragments {
        println!(
            "{:<12} {:<16} {:<14} {:<14} {:<10}",
            f.fragment_number,
            f.total_length,
            f.data_length,
            f.offset_field,
            f.mf_flag
        );
    }

    println!();

    // print detailed breakdown per fragment
    for f in &fragments {
        let byte_start = f.data_offset;
        let byte_end = f.data_offset + f.data_length - 1;

        println!("fragment {}", f.fragment_number);
        println!("  data sent    : {} bytes (bytes {} to {})", f.data_length, byte_start, byte_end);
        println!("  total size   : {} bytes", f.total_length);
        println!("  mf flag      : {} ({})", f.mf_flag, if f.mf_flag == 1 { "more fragments coming" } else { "last fragment" });
        println!("  offset field : {} (calculation: {} / 8 = {})", f.offset_field, f.data_offset, f.offset_field);
        println!();
    }

    // summary
    let total_data: usize = fragments.iter().map(|f| f.data_length).sum();
    let total_bytes: usize = fragments.iter().map(|f| f.total_length).sum();

    println!("summary");
    println!("  total fragments created  : {}", fragments.len());
    println!("  total payload reassembled: {} bytes", total_data);
    println!("  total bytes transmitted  : {} bytes (includes {} headers)", total_bytes, fragments.len() * ip_header);
    println!("  overhead from headers    : {} bytes", fragments.len() * ip_header);
}