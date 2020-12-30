Rust Power-Line Communication Management Library
================================================

This is an experimental Rust Library to discover PLC devices and manage them, along with a command line tool that uses the library.

Currently supported protocols:
* HomePlug AV 1.1/2.0 (Ethertype 0x88e1)
* Broadcom/Gigle Protocol (Ethertype 0x8912, "Mediaxtream"?)

These protocols are Layer 2 (Data link) and thus require the ability to send and receive raw ethernet frames. This library has a cross-platform layer to facilitate this in an efficient manner.

Currently supported OS':
* Linux (using AF_PACKET sockets)
* macOS (using /dev/bpf devices)

The core of the library is `no_std` to allow for use in low-resource devices (such as routers).


Qualcomm/Atheros Based Devices
------------------------------
These devices seem to implement all HomePlug AV H1 management frames documented in the HomePlug AV specifications. They also have a number of vendor extensions which are implemented in Qualcomm's [open-plc-utils](https://github.com/qca/open-plc-utils).

Tested with:
- QCA7420 - Netcomm NP505F (500Mbps HomePlug AV 1.1)


Broadcom/Gigle Devices
----------------------
While these devices seem to inter-operate at the power-line level with other HomePlug AV devices, their support of H1 management frames is limited. They appear to have most functionality exposed on ethertype 0x8912, which seems to be Gigle/Mediaxtream legacy.

Tested with:
- BCM60355 - D-Link DHP-601AV (1000Mbps HomePlug AV 2.0)


Resources
---------
[HomePlug Specifications](https://github.com/jbit/powerline/wiki/Documents)
[Qualcomm open-plc-utils](https://github.com/qca/open-plc-utils)


Command-line tool
-----------------

You may need to be root to use this tool!

Usage:
```
    powerline [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Increase verbosity

OPTIONS:
        --interface <interfaces>...    Select the interface(s) to discover with

SUBCOMMANDS:
    find        Find which interface a specific device is reachable
    help        Prints this message or the help of the given subcommand(s)
    scan        Discover and list devices
    set-name    Set the name of a device
```

To build and install to ~/.cargo/bin:
```
cargo install --example powerline --path .
```

Example output:
```
$ powerline 
eth0: [60:63:4c:11:22:33] HPAV2.0 Broadcom 3Ethers 'Lounge'
eth0: [60:63:4c:44:55:66] HPAV2.0 Broadcom 5Ethers 'Gateway'
eth0: [00:60:64:77:88:99] HPAV1.1 Qualcomm 8Ethers 'Upstairs'
```
The output shows that three HomePlug AV devices were found on the eth0 network interface.  
Two are HPAV2.0 devices from Broadcom. one is a HPAV1.1 device from Qualcomm.  
The XEthers field shows how many ethernet devices are bridge by the HPAV device.  
And the final text in quote marks is the device's friendly name.  
