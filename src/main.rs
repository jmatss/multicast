use std::io::stdin;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, UdpSocket};
use std::process::exit;
use std::time::Duration;
use std::{env, thread};

#[derive(PartialEq)]
enum Action {
    Send,
    Recv,
}

fn usage() {
    println!(
        "{0:-5} {1} {2}\n{3:-5} {1} {4}",
        "send:",
        env::args().nth(0).unwrap(),
        "<MCAST GROUP/IP> <PORT> [<AMOUNT> [<INTERVAL> [<SIZE>]]]",
        "recv:",
        "<MCAST GROUP/IP>"
    );
    println!("\n<AMOUNT> = amount of packets to send (default: 5)");
    println!("<INTERVAL> = delay between sent packets in ms (default: 1000 ms)");
    println!("<SIZE> = payload size per packet in bytes (default: 1 byte)");
}

// default values
const AMOUNT: &str = "5";
const INTERVAL: &str = "1000";
const SIZE: &str = "1";
const MAX_SIZE: usize = 1 << 16; // arbitrary value

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = if args.len() == 2 {
        Action::Recv
    } else if args.len() > 2 && args.len() <= 6 {
        Action::Send
    } else {
        usage();
        return;
    };

    let group = args
        .get(1)
        .unwrap()
        .parse::<IpAddr>()
        .expect("unable to parse multicast address");
    if !group.is_multicast() {
        panic!("specified address isn't a valid multicast address");
    }

    if action == Action::Recv {
        recv(group);
    } else {
        let port = args
            .get(2)
            .unwrap()
            .parse::<u16>()
            .expect("unable to parse port from string to integer");
        let amount = args
            .get(3)
            .unwrap_or(&AMOUNT.into())
            .parse::<u64>()
            .expect("unable to parse amount from string to integer");
        let interval = args
            .get(4)
            .unwrap_or(&INTERVAL.into())
            .parse::<u64>()
            .expect("unable to parse interval from string to integer");
        let size = args
            .get(5)
            .unwrap_or(&SIZE.into())
            .parse::<usize>()
            .expect("unable to parse payload size from string to integer");

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

        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(0);
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
            println!("Packet {} sent!", i + 1);
        }
    });

    t.join().unwrap();
    println!("DONE");
}

fn recv(group: IpAddr) {
    let socket: UdpSocket;
    match group {
        IpAddr::V4(ipv4) => {
            socket = crate::bind(&Ipv4Addr::UNSPECIFIED.into(), 0);
            socket
                .join_multicast_v4(&ipv4, &Ipv4Addr::UNSPECIFIED)
                .unwrap_or_else(|err| {
                    panic!(
                        "unable to join multicast group {}: {:?}",
                        group.to_string(),
                        err
                    )
                });
        }
        IpAddr::V6(ipv6) => {
            socket = crate::bind(&Ipv6Addr::UNSPECIFIED.into(), 0);
            socket.join_multicast_v6(&ipv6, 0).unwrap_or_else(|err| {
                panic!(
                    "unable to join multicast group {}: {:?}",
                    group.to_string(),
                    err
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
                    if n == 0 {
                        println!("sender closed socket");
                        exit(0);
                    } else {
                        println!(
                            "received packet with {} byte(s) from {}!",
                            n,
                            addr.to_string()
                        );
                    }
                }
                Err(e) => panic!("got error while receiving from socket: {:?}", e),
            }
        }
    });

    println!("Joined multicast group {}", group.to_string());
    println!("(press ENTER to exit)");
    let _ = stdin().read_line(&mut String::new());

    match group {
        IpAddr::V4(ipv4) => socket_clone
            .leave_multicast_v4(&ipv4, &Ipv4Addr::UNSPECIFIED)
            .unwrap_or_else(|err| {
                panic!(
                    "unable to leave multicast group {}: {:?}",
                    group.to_string(),
                    err
                )
            }),
        IpAddr::V6(ipv6) => socket_clone
            .leave_multicast_v6(&ipv6, 0)
            .unwrap_or_else(|err| {
                panic!(
                    "unable to leave multicast group {}: {:?}",
                    group.to_string(),
                    err
                )
            }),
    }
}
