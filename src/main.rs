extern crate pnet;
mod arp;

use arp::{listen_for_packets, send_packet};
use core::panic;
use pnet::datalink::{self, Channel, NetworkInterface};
use pnet::ipnetwork::IpNetwork;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn select_interface(ifaces: Vec<NetworkInterface>) -> Option<NetworkInterface> {
    //Printing to stdout in case no iterfaces are available
    if ifaces.is_empty() {
        println!("No interfaces available");
        return None;
    }

    println!("The available {} interfaces are:", ifaces.len());
    for (i, iface) in ifaces.iter().enumerate() {
        println!("{}\t{}", i, iface.description);
    }

    loop {
        //Asking user for input (choosing interface)
        println!("\nSelect interface (or \"q\" to quit)");
        io::stdout().flush().ok()?;

        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .map_err(|e| eprintln!("Failed to read input: {e}"))
            .ok()?;
        let input = buffer.trim();

        //Exits when user enters "q"
        if input == "q" {
            return None;
        }

        match input.parse::<usize>() {
            Ok(index) if index < ifaces.len() => return Some(ifaces[index].clone()),
            Ok(_) => eprintln!("Index out of range. Try again."),
            Err(_) => eprintln!("Invalid input. Please enter a number."),
        }
    }
}

fn main() {
    let interfaces = datalink::interfaces();
    let interface = match select_interface(interfaces) {
        Some(iface) => iface,
        None => {
            eprintln!("Did not find interface");
            return;
        }
    };

    let my_mac = match interface.mac {
        Some(mac) => mac,
        None => {
            eprintln!("No MAC address available!!");
            return;
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
    //dbg!(interface.clone());

    //Data channel
    let tunnel = datalink::channel(&interface, datalink::Config::default())
        .expect("Failed to create datalink channel");
    let (sender, recv) = match tunnel {
        Channel::Ethernet(tx, rx) => (tx, rx),
        _ => panic!("Unsupported channel type"),
    };
    let send_interface = interface.clone();

    let _reciever_thread = thread::spawn(move || {
        listen_for_packets(recv, my_ipv4_net);
    });
    let sender_thread = thread::spawn(move || {
        send_packet(sender, send_interface, my_ipv4_net, my_mac);
    });

    sender_thread.join().unwrap();
    println!("All packets have been sent\nWaiting 10 seconds for receiver!!");
    thread::sleep(Duration::from_secs(10));
    println!("Finished Scan");
}
