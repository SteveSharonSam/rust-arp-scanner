extern crate pnet;

use pnet::datalink::{self, NetworkInterface};


fn main() {
    let interfaces = datalink::interfaces();
    //println!("{:#?}", interfaces);
    let active_interfaces = |iface: &NetworkInterface| iface.is_up() && !iface.is_loopback();

    //println!("{:?}",active_interfaces(&interfaces.clone().into_iter().next().unwrap()));
    let active_ifaces: Vec<_> = interfaces.into_iter().filter(active_interfaces).collect();

    println!("{:#?}",active_ifaces);

}
