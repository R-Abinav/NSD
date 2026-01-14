pub fn ip_to_u32(ip: &str) -> u32{
    let parts: Vec<u32> = ip.split('.').map(|s| s.parse().unwrap()).collect();
    (parts[0] << 24) | (parts[1] << 16) | (parts[2] << 8) | parts[3]
}

pub fn u32_to_ip(ip: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        (ip >> 24) & 0xFF,
        (ip >> 16) & 0xFF,
        (ip >> 8) & 0xFF,
        ip & 0xFF
    )
}