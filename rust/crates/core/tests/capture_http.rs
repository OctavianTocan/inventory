//! The destination policy of `capture_http` against `lib/capture-http.mjs`: `publicAddress` over addresses around every
//! blocked-range boundary and `captureDestination` over generated URLs.

mod common;

use common::{Random, assert_same, calls};
use mm_core::capture_http::{capture_destination, public_address};
use mm_core::{Error, Value, arr};

const MODULE: &str = "lib/capture-http.mjs";

const SUBNETS: [([u8; 4], u32); 14] = [
    ([0, 0, 0, 0], 8), ([10, 0, 0, 0], 8), ([100, 64, 0, 0], 10), ([127, 0, 0, 0], 8), ([169, 254, 0, 0], 16), ([172, 16, 0, 0], 12),
    ([192, 0, 0, 0], 24), ([192, 0, 2, 0], 24), ([192, 88, 99, 0], 24), ([192, 168, 0, 0], 16), ([198, 18, 0, 0], 15),
    ([198, 51, 100, 0], 24), ([203, 0, 113, 0], 24), ([224, 0, 0, 0], 3),
];

fn dotted(value: u32) -> String {
    let [a, b, c, d] = value.to_be_bytes();
    format!("{a}.{b}.{c}.{d}")
}

fn v4_addresses(random: &mut Random) -> Vec<String> {
    let mut out = Vec::new();
    for (octets, prefix) in SUBNETS {
        let first = u32::from_be_bytes(octets);
        let last = first | (u32::MAX >> prefix);
        for value in [first.wrapping_sub(1), first, first + 1, last.wrapping_sub(1), last, last.wrapping_add(1)] {
            out.push(dotted(value));
        }
    }
    out.extend(["8.8.8.8", "1.1.1.1", "255.255.255.255", "0.0.0.0", "01.2.3.4", "1.2.3", "1.2.3.4.5", "256.1.1.1", "1.2.3.-4", "", " 1.2.3.4",
        "1.2.3.4 ", "0x7f.0.0.1", "127.1", "1e2.1.1.1", "1.2.3.04", "08.8.8.8", "+1.2.3.4", "١.٢.٣.٤"].map(str::to_owned));
    out.extend((0..400).map(|_| dotted(random.next() as u32)));
    out.extend((0..200).map(|_| dotted(u32::from_be_bytes([random.below(256) as u8, random.below(4) as u8, random.below(256) as u8, random.below(256) as u8]))));
    out
}

fn groups(random: &mut Random, prefix: &[u16], count: usize) -> String {
    let mut parts: Vec<String> = prefix.iter().map(|group| format!("{group:x}")).collect();
    parts.extend((0..count).map(|_| format!("{:x}", random.below(65536))));
    parts.join(":")
}

fn v6_addresses(random: &mut Random) -> Vec<String> {
    let mut out: Vec<String> = [
        "::", "::1", "::2", "1::", "2000::", "2001::", "2001:1ff:ffff:ffff:ffff:ffff:ffff:ffff", "2001:200::1", "2001:db8::", "2001:db7:ffff::1",
        "2001:db9::1", "2002::", "2002:ffff::1", "2003::1", "3fff::", "3fff:fff:ffff::1", "3fff:1000::1", "3ffe::1", "2606:4700::1111", "2a00:1450::1",
        "3fff:0fff:ffff:ffff:ffff:ffff:ffff:ffff", "1fff::1", "4000::1", "fc00::1", "fe80::1", "fe80::1%eth0", "2606:4700::1%1", "ff02::1",
        "::ffff:8.8.8.8", "::ffff:808:808", "::8.8.8.8", "::1.2.3.4", "2606:4700::1.2.3.4", "64:ff9b::808:808", "2606:4700:0:0:0:0:0:1",
        "2606:4700::0:1", "2606:4700:::1", ":2606::1", "2606::1:", "2606:4700::1::2", "1:2:3:4:5:6:7:8", "1:2:3:4:5:6:7:8:9", "1:2:3:4:5:6:7::",
        "::1:2:3:4:5:6:7", "::1:2:3:4:5:6:7:8", "2606:4700::g", "2606:47000::1", "2606:4700::1%", "2606:4700::1%a b", "[2606:4700::1]", "2606:4700::01",
        "2606:4700::0001", "2606:4700::00001", "ABCD::EF01", "2001:DB8::1", "::FFFF:1.2.3.4", "2606:4700::1.2.3", "2606:4700::256.1.1.1",
        "2606:4700::01.2.3.4",
    ]
    .map(str::to_owned)
    .to_vec();
    for _ in 0..500 {
        let first = [0x2000 + random.below(0x2000) as u16, 0x2001, 0x2002, 0x3fff, 0x2606, 0x0, 0xfe80, 0x2a00][random.below(8) as usize];
        let second = [0, 0x1ff, 0x200, 0xdb8, 0xfff, 0x1000, random.below(65536) as u16][random.below(7) as usize];
        let count = 1 + random.below(6) as usize;
        let text = groups(random, &[first, second], count);
        out.push(if random.below(2) == 0 { format!("{text}::") } else { format!("{text}::{:x}", random.below(65536)) });
    }
    out
}

fn urls(random: &mut Random) -> Vec<String> {
    let schemes = ["http://", "https://", "HTTP://", "ftp://", "//", "", "javascript:", "http:/", "https:\\\\"];
    let users = ["", "", "", "user@", "user:pw@", ":@", "a%40b@"];
    let hosts = [
        "example.com", "EXAMPLE.com", "example.com.", "localhost", "localhost.", "LOCALHOST", "a.localhost", "a.localhost.", "x.y.localhost", "127.0.0.1",
        "127.1", "127.0.1", "2130706433", "0x7f000001", "0x7f.1", "0177.0.0.1", "017700000001", "[::1]", "[::]", "[::ffff:127.0.0.1]", "[::ffff:8.8.8.8]",
        "[2606:4700::1111]", "[fe80::1]", "[2001:db8::1]", "[3fff::1]", "[2002::1]", "8.8.8.8", "8.8.8.8.", "8.8.8", "8.8.8.8:80", "10.0.0.1", "10.1", "100.64.0.1",
        "169.254.169.254", "172.16.0.1", "172.32.0.1", "192.168.1.1", "192.0.0.8", "198.18.0.1", "224.0.0.1", "255.255.255.255", "0.0.0.0", "0", "1.1.1.1",
        "access.mymind.com", "ACCESS.mymind.com", "access.mymind.com.", "www.mymind.com", "mymind.com", "xn--e1afmkfd.xn--p1ai", "ex ample.com", "exa%6dple.com",
        "a_b.example", "1.2.3.4", "1.2.3.4.5", "0x08.8.8.8", "8.8.8.0x8", "", "..", "a..b", "[::1", "::1]",
    ];
    let ports = ["", "", ":80", ":443", ":8080", ":0", ":65535", ":65536", ":abc", ":"];
    let paths = ["", "/", "/a/b?c=d#e", "?q", "#f", "/%zz", "/a b"];
    let mut out = Vec::new();
    for host in hosts {
        for scheme in ["http://", "https://"] {
            out.push(format!("{scheme}{host}/"));
        }
    }
    for _ in 0..900 {
        out.push(format!(
            "{}{}{}{}{}",
            schemes[random.below(schemes.len() as u64) as usize],
            users[random.below(users.len() as u64) as usize],
            hosts[random.below(hosts.len() as u64) as usize],
            ports[random.below(ports.len() as u64) as usize],
            paths[random.below(paths.len() as u64) as usize]
        ));
    }
    out.extend(["", " ", "not a url", "http://", "https://?", "  https://example.com/  ", "https://example.com/\n", "https://exam\tple.com/", "HTTPS://EXAMPLE.COM/A"].map(str::to_owned));
    out
}

#[test]
fn public_address_matches_node() {
    let mut random = Random(0x9e37_79b9);
    let mut all = v4_addresses(&mut random);
    all.extend(v6_addresses(&mut random));
    let requests: Vec<(&str, Vec<Value>)> = all.iter().map(|address| ("publicAddress", vec![Value::from(address.as_str())])).collect();
    let answers = calls(MODULE, &requests);
    let mut public = 0;
    for (address, answer) in all.iter().zip(&answers) {
        let rust: Result<Value, Error> = Ok(Value::Bool(public_address(address)));
        assert_same(&rust, answer, address);
        public += usize::from(public_address(address));
    }
    assert!(public > 100 && public < all.len() - 100, "{public} of {} public", all.len());
    println!("publicAddress cases: {}", all.len());
}

#[test]
fn capture_destination_matches_node() {
    let mut random = Random(0x51ed_270b);
    let all = urls(&mut random);
    let mut requests = Vec::new();
    for url in &all {
        for offline in [false, true] {
            requests.push(("captureDestination", vec![Value::from(url.as_str()), Value::Bool(offline)]));
        }
    }
    let answers = calls(MODULE, &requests);
    let mut accepted = 0;
    for ((request, answer), index) in requests.iter().zip(&answers).zip(0..) {
        let url = request.1[0].as_str().unwrap();
        let offline = index % 2 == 1;
        let rust = capture_destination(url, offline).map(Value::from);
        accepted += usize::from(rust.is_ok());
        assert_same(&rust, answer, &format!("{url:?} offline={offline}"));
    }
    assert!(accepted > 50, "{accepted} accepted");
    println!("captureDestination cases: {}", requests.len());
    let _ = arr![];
}
