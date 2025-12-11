extern crate pnet;
mod arp;
mod cli;
mod interface;

use crate::interface::display;
use arp::{listen_for_packets, send_packet};
use clap::Parser;
use cli::Cli;
use core::panic;
use pnet::datalink::{self, Channel};
use pnet::ipnetwork::IpNetwork;
use std::any::Any;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Send + Any>> {
    let input = Cli::parse();
    let interfaces = match input.list {
        false => datalink::interfaces(),
        true => {
            display();
            return Ok(());
        }
    };
    let interface = interfaces[input.iface].clone();

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

    match sender_thread.join() {
        Ok(it) => it,
        Err(err) => return Err(err),
    };
    println!("All packets have been sent\nWaiting 10 seconds for receiver!!");
    thread::sleep(Duration::from_secs(10));
    println!("Finished Scan");
    Ok(())
}
