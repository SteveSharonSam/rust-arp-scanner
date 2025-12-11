# ARP Scanner

A simple network scanner written in Rust that discovers active devices on your local network using ARP (Address Resolution Protocol) requests.

## Security Note

This tool is intended for network administration and educational purposes only. Only scan networks you own or have explicit permission to scan. Unauthorized network scanning may be illegal in your jurisdiction.

## Overview

This tool scans your local network by sending ARP requests to all IP addresses in your subnet and listening for ARP replies. When a device responds, it displays the device's IP and MAC address.

## Features

- Interactive interface selection
- Automatic subnet scanning based on your network configuration
- Real-time device discovery
- Displays both IP and MAC addresses of discovered devices

## Prerequisites

- Rust (latest stable version)
- Administrative/root privileges (required for raw packet operations)
- A network interface with an IPv4 address
- Windows users: Npcap driver must be installed. See [github.com/libpnet/libpnet#windows](https://github.com/libpnet/libpnet#windows).

## Dependencies

This project uses the `pnet` crate for low-level network operations:

```toml
[dependencies]
pnet = "0.35"
```

## Installation

1. Clone the repository:
```bash
git clone https://github.com/SteveSharonSam/rust-arp-scanner.git
cd arp-scanner
```

2. Build the project:
```bash
cargo build --release
```

## Usage

Run the scanner with elevated privileges:

### Linux/macOS:
```bash
sudo ./target/release/arp-scanner
```

### Windows (Administrator):
```bash
.\target\release\arp-scanner.exe
```

### How it works:

1. The program lists all available network interfaces
2. Select the interface you want to scan (typically your active network connection)
3. The scanner will:
   - Send ARP requests to all IPs in your subnet
   - Listen for ARP replies
   - Display discovered devices in real-time
4. After scanning all addresses, it waits 10 seconds for final responses

## Example Output

```
MAC: aa:bb:cc:dd:ee:ff
IP: 192.168.1.100/24
Devices found:
Status ONLINE
IP addr:        192.168.1.1
Mac addr:       11:22:33:44:55:66
----------------------
Status ONLINE
IP addr:        192.168.1.50
Mac addr:       aa:bb:cc:dd:ee:ff
----------------------
```

## Known Limitations

- Only scans IPv4 networks (IPv6 not supported)
- Requires root/administrator privileges
- May not detect devices with strict firewall rules
- Fixed timing parameters may need adjustment for larger networks
- Scanner only sends one ARP request per IP (may miss some devices)

## To do 
- Proper error propagation
- Prevent panics
- Proper Input
- Proper Output

## Troubleshooting
**Windows: "Failed to create datalink channel"**
- Install the Npcap driver from https://npcap.com/
- During installation, make sure to check "Install Npcap in WinPcap API-compatible Mode"
- Restart your computer after installation

**"No MAC address available!!"**
- Ensure your network interface is properly configured
- Check that you have an active network connection

**"Failed to create datalink channel"**
- Make sure you're running with root/administrator privileges
- Verify that your operating system supports raw sockets

**No devices detected**
- Try increasing the wait time at the end
- Check your firewall settings
- Ensure you selected the correct network interface

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
