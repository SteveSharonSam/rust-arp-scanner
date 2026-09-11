use pnet::datalink::{DataLinkReceiver, DataLinkSender, NetworkInterface};
use pnet::ipnetwork::Ipv4Network;
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::{MutablePacket, Packet};
use pnet::util::MacAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Size of a raw ARP packet: 8-byte header + 2 * (6-byte hw addr + 4-byte proto addr).
const ARP_PACKET_LEN: usize = 28;
/// 14-byte Ethernet header + the ARP payload above.
const ETHERNET_FRAME_LEN: usize = 14 + ARP_PACKET_LEN;
/// How long the receiver keeps listening for straggler replies after the
/// sender has finished broadcasting every request.
const RECEIVE_GRACE_PERIOD: Duration = Duration::from_secs(2);

pub fn send_packet(
    mut tx: Box<dyn DataLinkSender>,
    interface: NetworkInterface,
    sender_ip: Ipv4Network,
    sender_macaddr: MacAddr,
) {
    // pnet's `send_to` requires an owned NetworkInterface per call, so clone
    // it once up front rather than once per packet. We already own
    // `interface` here (it was moved in, not borrowed), but send_to still
    // needs its own copy on every call.
    let send_interface = Some(interface.clone());

    for target_ip in sender_ip.iter() {
        if target_ip == sender_ip.ip() {
            continue;
        }

        let mut arp_buf = [0u8; ARP_PACKET_LEN];
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

        let mut ethernet_buf = [0u8; ETHERNET_FRAME_LEN];
        let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buf).unwrap();

        ethernet_packet.set_destination(MacAddr::broadcast());
        ethernet_packet.set_source(sender_macaddr);
        ethernet_packet.set_ethertype(EtherTypes::Arp);
        ethernet_packet.set_payload(arp_packet.packet_mut());

        if let Some(Err(e)) = tx.send_to(
            ethernet_packet.to_immutable().packet(),
            send_interface.clone(),
        ) {
            eprintln!("Failed to send ARP request to {target_ip}: {e}");
        }
    }
}

pub fn listen_for_packets(
    mut rx: Box<dyn DataLinkReceiver>,
    ipv4_net: Ipv4Network,
    sending_done: &AtomicBool,
) {
    let mut done_at: Option<Instant> = None;

    loop {
        match rx.next() {
            Ok(buffer) => {
                if let Some(ethernet_packet) = EthernetPacket::new(buffer) {
                    if ethernet_packet.get_ethertype() == EtherTypes::Arp {
                        if let Some(arp_packet) = ArpPacket::new(ethernet_packet.payload()) {
                            if arp_packet.get_operation() == ArpOperations::Reply
                                && arp_packet.get_target_proto_addr() == ipv4_net.ip()
                            {
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
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock
                    && e.kind() != std::io::ErrorKind::TimedOut
                {
                    eprintln!("Error reading packet: {e}");
                }
            }
        }

        if sending_done.load(Ordering::Relaxed) {
            let elapsed_since_done = done_at.get_or_insert_with(Instant::now);
            if elapsed_since_done.elapsed() > RECEIVE_GRACE_PERIOD {
                break;
            }
        }
    }
}
