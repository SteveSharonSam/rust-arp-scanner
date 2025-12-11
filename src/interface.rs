use pnet::datalink;

pub(crate) fn display() {
    let ifaces = datalink::interfaces();
    if ifaces.is_empty() {
        eprintln!("No interfaces available");
    }

    println!("The available {} interfaces are:", ifaces.len());
    println!("{:<6} {:<45} {:<6}", "Index", "Name", "Status");
    for (i, iface) in ifaces.iter().enumerate() {
        println!("{:<6} {:<45} {:<6}", i, iface.description, iface.is_up());
    }
}
