use std::collections::{HashMap, HashSet};
use std::fmt;

//crc32 ieee 802.3
struct Crc32 {
    table: [u32; 256],
}

impl Crc32 {
    const POLYNOMIAL: u32 = 0xEDB88320;

    fn new() -> Self {
        let mut table = [0u32; 256];
        for i in 0..256 {
            let mut crc = i as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ Self::POLYNOMIAL;
                } else {
                    crc >>= 1;
                }
            }
            table[i] = crc;
        }
        Self { table }
    }

    fn compute(&self, data: &[u8]) -> u32 {
        let mut crc = 0xFFFFFFFF;
        for &byte in data {
            let index = ((crc ^ byte as u32) & 0xFF) as usize;
            crc = (crc >> 8) ^ self.table[index];
        }
        !crc
    }

    fn hash_to_index(&self, mac: &MacAddress, bits: u8) -> usize {
        let crc = self.compute(&mac.0);
        let shift = 32 - bits;
        ((crc >> shift) as usize) & ((1 << bits) - 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MacAddress([u8; 6]);

impl MacAddress {
    fn from_multicast_ip(ip: &IpAddress) -> Self {
        let b = ip.0;
        Self([0x01, 0x00, 0x5E, b[1] & 0x7F, b[2], b[3]])
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct IpAddress([u8; 4]);

impl IpAddress {
    fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }

    fn is_multicast(&self) -> bool {
        self.0[0] >= 224 && self.0[0] <= 239
    }
}

impl fmt::Display for IpAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

#[derive(Clone)]
struct MulticastPacket {
    dst_ip: IpAddress,
    dst_mac: MacAddress,
}

impl MulticastPacket {
    fn new(dst_ip: IpAddress) -> Self {
        let dst_mac = MacAddress::from_multicast_ip(&dst_ip);
        Self { dst_ip, dst_mac }
    }
}

struct HardwareHashTable {
    bits: Vec<bool>,
    size_bits: u8,
    crc: Crc32,
}

impl HardwareHashTable {
    fn new(size_bits: u8) -> Self {
        Self {
            bits: vec![false; 1 << size_bits],
            size_bits,
            crc: Crc32::new(),
        }
    }

    fn add_mac(&mut self, mac: &MacAddress) {
        let idx = self.crc.hash_to_index(mac, self.size_bits);
        self.bits[idx] = true;
    }

    fn check_mac(&self, mac: &MacAddress) -> bool {
        let idx = self.crc.hash_to_index(mac, self.size_bits);
        self.bits[idx]
    }
}

struct SoftwareFilter {
    subscribed: HashSet<IpAddress>,
}

impl SoftwareFilter {
    fn new() -> Self {
        Self {
            subscribed: HashSet::new(),
        }
    }

    fn subscribe(&mut self, ip: IpAddress) {
        self.subscribed.insert(ip);
    }

    fn is_subscribed(&self, ip: &IpAddress) -> bool {
        self.subscribed.contains(ip)
    }
}

#[derive(Default)]
struct SimulationStats {
    total: usize,
    hw_dropped: usize,
    hw_passed: usize,
    sw_accepted: usize,
    sw_dropped: usize,
}

struct MulticastFilterSimulator {
    hw: HardwareHashTable,
    sw: SoftwareFilter,
    stats: SimulationStats,
    mac_to_ips: HashMap<MacAddress, Vec<IpAddress>>,
}

impl MulticastFilterSimulator {
    fn new(bits: u8) -> Self {
        Self {
            hw: HardwareHashTable::new(bits),
            sw: SoftwareFilter::new(),
            stats: SimulationStats::default(),
            mac_to_ips: HashMap::new(),
        }
    }

    fn subscribe(&mut self, ip: IpAddress) {
        assert!(ip.is_multicast());
        self.sw.subscribe(ip);
        let mac = MacAddress::from_multicast_ip(&ip);
        self.hw.add_mac(&mac);
        self.mac_to_ips.entry(mac).or_insert_with(Vec::new).push(ip);
    }

    fn process(&mut self, pkt: MulticastPacket) {
        self.stats.total += 1;

        if !self.hw.check_mac(&pkt.dst_mac) {
            self.stats.hw_dropped += 1;
            return;
        }

        self.stats.hw_passed += 1;

        if self.sw.is_subscribed(&pkt.dst_ip) {
            self.stats.sw_accepted += 1;
        } else {
            self.stats.sw_dropped += 1;
        }
    }
}

fn generate_well_known_addresses() -> Vec<IpAddress> {
    vec![
        IpAddress::new(224, 0, 0, 1),
        IpAddress::new(224, 0, 0, 2),
        IpAddress::new(224, 0, 0, 5),
        IpAddress::new(224, 0, 0, 6),
        IpAddress::new(224, 0, 0, 9),
    ]
}

fn main() {
    let mut sim = MulticastFilterSimulator::new(4);

    let subs = vec![
        IpAddress::new(224, 0, 0, 1),
        IpAddress::new(224, 0, 0, 5),
        IpAddress::new(224, 0, 0, 251),
        IpAddress::new(239, 192, 1, 1),
        IpAddress::new(239, 192, 2, 2),
    ];

    println!("multicast filter simulation");
    println!();
    println!("hardware hash table size: {} bits ({} entries)", 4, 1 << 4);
    println!();

    println!("subscribed multicast groups:");
    for ip in &subs {
        let mac = MacAddress::from_multicast_ip(ip);
        println!("  {} -> {}", ip, mac);
        sim.subscribe(*ip);
    }
    println!();

    let mut packets = Vec::new();

    for ip in &subs {
        for _ in 0..20 {
            packets.push(MulticastPacket::new(*ip));
        }
    }

    let others = vec![
        IpAddress::new(224, 0, 0, 2),
        IpAddress::new(224, 0, 0, 9),
        IpAddress::new(224, 0, 1, 1),
        IpAddress::new(239, 100, 1, 1),
        IpAddress::new(239, 100, 2, 2),
        IpAddress::new(238, 50, 50, 50),
        IpAddress::new(224, 1, 1, 1),
        IpAddress::new(224, 2, 2, 2),
        IpAddress::new(225, 1, 1, 1),
        IpAddress::new(230, 5, 5, 5),
    ];

    for ip in &others {
        for _ in 0..30 {
            packets.push(MulticastPacket::new(*ip));
        }
    }

    println!("processing {} packets...", packets.len());
    println!();

    for pkt in packets {
        sim.process(pkt);
    }

    println!("simulation results:");
    println!("  total packets: {}", sim.stats.total);
    println!("  hardware dropped: {}", sim.stats.hw_dropped);
    println!("  hardware passed: {}", sim.stats.hw_passed);
    println!("  software accepted: {}", sim.stats.sw_accepted);
    println!("  software dropped: {}", sim.stats.sw_dropped);
    println!();

    let hw_filter_rate = (sim.stats.hw_dropped as f64 / sim.stats.total as f64) * 100.0;
    let false_positive_rate = (sim.stats.sw_dropped as f64 / sim.stats.hw_passed as f64) * 100.0;

    println!("performance metrics:");
    println!("  hardware filtering ratio: {:.2}%", hw_filter_rate);
    println!("  false positive rate: {:.2}%", false_positive_rate);
    println!();

    println!("hash collision analysis:");
    for (mac, ips) in &sim.mac_to_ips {
        if ips.len() > 1 {
            println!("  collision at mac {}", mac);
            for ip in ips {
                println!("    {}", ip);
            }
        }
    }
    
    if sim.stats.sw_dropped > 0 {
        println!();
        println!("false positive examples (unsubscribed traffic leaked through hw filter):");
        println!("  {} packets from unsubscribed groups passed hardware filter", sim.stats.sw_dropped);
        println!("  these share hash indices with subscribed groups");
    }

    let crc = Crc32::new();
    println!();
    println!("well-known multicast addresses:");
    for ip in generate_well_known_addresses().iter() {
        let mac = MacAddress::from_multicast_ip(ip);
        let hash_index = crc.hash_to_index(&mac, 4);
        println!("  {} -> {} (hash index: {})", ip, mac, hash_index);
    }
}