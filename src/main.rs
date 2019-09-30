extern crate chrono;
extern crate getopts;

use chrono::prelude::*;
use getopts::Options;
use std::io::stdin;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, UdpSocket};
use std::time::Duration;
use std::{env, thread};

#[derive(PartialEq)]
enum Action {
    Send,
    Recv,
}

fn usage(opts: &Options) {
    let brief = format!(
        "{0:-7} {1} {2}\n{3:-7} {1} {4}",
        "usage:",
        env::args().nth(0).unwrap(),
        "send <MCAST IP> <PORT> [options]",
        "",
        "recv <MCAST IP> <PORT>"
    );
    print!("{}", opts.usage(&brief));
}

// default values
const AMOUNT: &str = "5";
const INTERVAL: &str = "1000";
const SIZE: &str = "1";
const MAX_SIZE: usize = 1 << 16; // arbitrary value

fn main() {
    let args: Vec<String> = env::args().collect();

    let (a, i, s) = ("a", "i", "s");
    let mut opts = Options::new();
    opts.optopt(a, "amount", "amount of packets to send (default: 5)", "")
        .optopt(
            i,
            "interval",
            "delay between sent packets in ms (default: 1000 ms)",
            "",
        )
        .optopt(
            s,
            "size",
            "payload size per packet in bytes (default: 1 byte)",
            "",
        );

    if args.len() < 4 {
        usage(&opts);
        return;
    }

    let action_string = args.get(1).unwrap();
    if action_string.is_empty() {
        usage(&opts);
        return;
    }

    let action = if action_string == "recv" || action_string == "r" {
        Action::Recv
    } else if action_string == "send" || action_string == "s" {
        Action::Send
    } else {
        usage(&opts);
        return;
    };

    let group = args
        .get(2)
        .unwrap()
        .parse::<IpAddr>()
        .expect("unable to parse multicast address");
    if !group.is_multicast() {
        panic!("specified address isn't a valid multicast address");
    }

    let port = args
        .get(3)
        .unwrap()
        .parse::<u16>()
        .expect("unable to parse port from string to integer");

    if action == Action::Recv {
        recv(group, port);
    } else {
        // action == Action::Send
        let matches = match opts.parse(&args[4..]) {
            Ok(m) => m,
            Err(e) => panic!(e.to_string()),
        };

        let amount = matches
            .opt_str(a)
            .unwrap_or(AMOUNT.into())
            .parse::<u64>()
            .expect("unable to parse amount from string to integer");
        let interval = matches
            .opt_str(i)
            .unwrap_or(INTERVAL.into())
            .parse::<u64>()
            .expect("unable to parse interval from string to integer");
        let size = matches
            .opt_str(s)
            .unwrap_or(SIZE.into())
            .parse::<usize>()
            .expect("unable to parse size from string to integer");

        send(group, port, amount, interval, size);
    }
}

fn bind(group: &IpAddr, port: u16) -> UdpSocket {
    let group_string = if group.is_ipv6() {
        format!("[{}]", group.to_string())
    } else {
        group.to_string()
    };

    UdpSocket::bind(format!("{}:{}", group_string, port.to_string())).unwrap_or_else(|err| {
        panic!(
            "unable to bind to address & port {}:{}: {:?}",
            group_string,
            port.to_string(),
            err
        )
    })
}

fn send(group: IpAddr, port: u16, amount: u64, interval: u64, size: usize) {
    if port == 0 || amount == 0 || interval == 0 || size == 0 {
        panic!("a given input is == 0");
    } else if size > MAX_SIZE {
        panic!(
            "size > MAX_SIZE ({} > {}): change hardcoded const \"MAX_SIZE\" in code to increase limit",
            size, MAX_SIZE
        );
    }

    let socket = if group.is_ipv4() {
        crate::bind(&Ipv4Addr::UNSPECIFIED.into(), 0)
    } else {
        crate::bind(&Ipv6Addr::UNSPECIFIED.into(), 0)
    };

    let t = thread::spawn(move || {
        let group_string = if group.is_ipv6() {
            format!("[{}]", group.to_string())
        } else {
            group.to_string()
        };

        let mut data: Vec<u8> = Vec::with_capacity(size);
        for i in 0..size {
            data.push((i % 255) as u8);
        }

        let sleep_interval = Duration::from_millis(interval);
        for i in 0..amount {
            if i != 0 {
                thread::sleep(sleep_interval);
            }
            socket
                .set_multicast_ttl_v4(255)
                .expect("unable to set multicast ttl");
            socket
                .send_to(&data, format!("{}:{}", group_string, port.to_string()))
                .unwrap_or_else(|err| {
                    panic!(
                        "unable to send packets to {}:{}: {:?}",
                        group_string,
                        port.to_string(),
                        err
                    )
                });

            println!(
                "{} : sent {} byte(s) to {}:{}",
                Local::now().format("%H:%M:%S%.3f (%Z)"),
                size,
                group_string,
                port.to_string()
            );
        }
    });

    t.join().unwrap();
}

fn recv(group: IpAddr, port: u16) {
    let socket: UdpSocket;
    match group {
        IpAddr::V4(ip) => {
            socket = crate::bind(&Ipv4Addr::UNSPECIFIED.into(), port);
            socket
                .join_multicast_v4(&ip, &Ipv4Addr::UNSPECIFIED)
                .unwrap_or_else(|err| {
                    panic!(
                        "unable to join multicast group {}:{}: {:?}",
                        group.to_string(),
                        err,
                        port
                    )
                });
        }
        IpAddr::V6(ip) => {
            socket = crate::bind(&Ipv6Addr::UNSPECIFIED.into(), port);
            socket.join_multicast_v6(&ip, 0).unwrap_or_else(|err| {
                panic!(
                    "unable to join multicast group {}:{}: {:?}",
                    group.to_string(),
                    err,
                    port
                )
            });
        }
    }

    let socket_clone = socket.try_clone().expect("unable to clone socket");

    thread::spawn(move || {
        let mut buf = [0; MAX_SIZE];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((n, addr)) => {
                    println!(
                        "{} : received {} byte(s) from {}",
                        Local::now().format("%H:%M:%S%.3f (%Z)"),
                        n,
                        addr.to_string()
                    );
                }
                Err(e) => panic!("got error while receiving from socket: {:?}", e),
            }
        }
    });

    println!(
        "Joined multicast group {} (press ENTER to exit)",
        group.to_string(),
    );

    println!(
        "Listening on socket {}",
        socket_clone
            .local_addr()
            .expect("unable to get local address from socket"),
    );
    let _ = stdin().read_line(&mut String::new());

    match group {
        IpAddr::V4(ip) => socket_clone
            .leave_multicast_v4(&ip, &Ipv4Addr::UNSPECIFIED)
            .unwrap_or_else(|err| {
                panic!(
                    "unable to leave multicast group {}: {:?}",
                    group.to_string(),
                    err
                )
            }),
        IpAddr::V6(ip) => socket_clone
            .leave_multicast_v6(&ip, 0)
            .unwrap_or_else(|err| {
                panic!(
                    "unable to leave multicast group {}: {:?}",
                    group.to_string(),
                    err
                )
            }),
    }
}
