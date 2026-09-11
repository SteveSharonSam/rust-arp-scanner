extern crate pnet;
mod arp;
//mod cli;
mod interface;

use crate::interface::display;
use arp::{listen_for_packets, send_packet};
use core::panic;
use pnet::datalink::{self, Channel};
use pnet::ipnetwork::IpNetwork;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    let Some(interfaces) = display() else {
        println!("Quitting");
        return Ok(());
    };
    print!("Select interface: ");

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let idx: usize = input
        .trim()
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid interface number"))?;
    let interface = interfaces
        .into_iter()
        .nth(idx)
        .expect("Index should be within bounds");

    let my_mac = match interface.mac {
        Some(mac) => mac,
        None => {
            eprintln!("No MAC address available!!");
            return Ok(());
        }
    };

    let my_ipv4_net = interface
        .ips
        .iter()
        .filter_map(|ipnetwork| match ipnetwork {
            IpNetwork::V4(v4) => Some(*v4),
            _ => None,
        })
        .next()
        .expect("No ip address found");

    println!("MAC: {my_mac}");
    println!("IP: {my_ipv4_net}");
    println!("Devices found:");

    let mut config = datalink::Config::default();
    config.read_timeout = Some(Duration::from_millis(200));

    let tunnel = datalink::channel(&interface, config).expect("Failed to create datalink channel");
    let (sender, recv) = match tunnel {
        Channel::Ethernet(tx, rx) => (tx, rx),
        _ => panic!("Unsupported channel type"),
    };

    //a flag shared between sending and recieving thread
    //flag set to true by sender thread after which recieving thread stops listening
    let sending_done = AtomicBool::new(false);
    thread::scope(|s| {
        let receiver_thread = s.spawn(|| {
            listen_for_packets(recv, my_ipv4_net, &sending_done);
        });

        let sender_thread = s.spawn(|| {
            send_packet(sender, interface, my_ipv4_net, my_mac);
            sending_done.store(true, Ordering::Relaxed);
        });

        sender_thread.join().expect("Sender thread panicked");
        receiver_thread.join().expect("Receiver thread panicked");
    });

    println!("All packets have been sent");
    println!("Finished Scan");
    Ok(())
}
