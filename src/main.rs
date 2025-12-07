extern crate pnet;

use core::panic;
use pnet::datalink::{self, Channel, DataLinkReceiver, DataLinkSender, NetworkInterface};
use pnet::ipnetwork::{IpNetwork, Ipv4Network};
use pnet::packet::arp::{ArpHardwareTypes, MutableArpPacket};
use pnet::packet::arp::{ArpOperations, ArpPacket};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::{MutablePacket, Packet};
use pnet::util::MacAddr;
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

fn send_packet(
    mut tx: Box<dyn DataLinkSender>,
    interface: NetworkInterface,
    sender_ip: Ipv4Network,
    sender_macaddr: MacAddr,
) {
    for target_ip in sender_ip.iter() {
        if target_ip == sender_ip.ip() {
            continue;
        }
        for _ in 0..1 {
            //arp packet
            let mut arp_buf = [0u8; 28];
            let mut arp_packet = MutableArpPacket::new(&mut arp_buf).unwrap();

            arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
            arp_packet.set_protocol_type(EtherTypes::Ipv4);
            arp_packet.set_hw_addr_len(6);
            arp_packet.set_operation(ArpOperations::Request);
            arp_packet.set_proto_addr_len(4);
            arp_packet.set_sender_hw_addr(sender_macaddr);
            arp_packet.set_sender_proto_addr(sender_ip.ip());
            arp_packet.set_target_hw_addr(MacAddr::zero());
            arp_packet.set_target_proto_addr(target_ip);

            //ethernet packet
            let mut ethernet_buf = [0u8; 42];
            let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buf).unwrap();

            ethernet_packet.set_destination(MacAddr::broadcast());
            ethernet_packet.set_source(sender_macaddr);
            ethernet_packet.set_ethertype(EtherTypes::Arp);
            ethernet_packet.set_payload(arp_packet.packet_mut());

            tx.send_to(
                &ethernet_packet.to_immutable().packet(),
                Some(interface.clone()),
            );
            thread::sleep(Duration::from_millis(20));
        }
        thread::sleep(Duration::from_millis(60));
    }
}

fn listen_for_packets(mut rx: Box<dyn DataLinkReceiver>, ipv4_net: Ipv4Network) {
    loop {
        let arp_buffer = match rx.next() {
            Ok(buffer) => buffer,
            Err(_) => continue,
        };
        let ethernet_packet = EthernetPacket::new(arp_buffer).unwrap();

        if ethernet_packet.get_ethertype() == EtherTypes::Arp {
            let arp_packet = ArpPacket::new(ethernet_packet.payload()).unwrap();
            if arp_packet.get_operation() == ArpOperations::Reply {
                if arp_packet.get_target_proto_addr() == ipv4_net.ip() {
                    println!("Status ONLINE");
                    println!(
                        "IP addr:\t{} \nMac addr:\t{}",
                        arp_packet.get_sender_proto_addr(),
                        arp_packet.get_sender_hw_addr()
                    );
                    println!("----------------------");
                }
            }
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
