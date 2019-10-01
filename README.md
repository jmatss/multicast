Multicast sender/receiver for ipv4 &amp; ipv6.

```
usage:  multicast send <MCAST IP> <PORT> [options]
        multicast recv <MCAST IP> <PORT>

Options:
    -a, --amount        amount of packets to send (default: 5)
    -i, --interval      delay between sent packets in ms (default: 1000 ms)
    -s, --size          payload size per packet in bytes (default: 1 byte)
    -t, --ttl           time to live for packets (default: 255)
```
Examples:
```
$ ./multicast recv 224.0.2.15 1337
Joined multicast group 224.0.2.15 (press ENTER to exit)
Listening on socket 0.0.0.0:1337
22:30:50.132 (+02:00) : received 12 byte(s) from 192.168.10.2:51423
22:30:51.158 (+02:00) : received 12 byte(s) from 192.168.10.2:51423
22:30:52.154 (+02:00) : received 12 byte(s) from 192.168.10.2:51423
```
```
$ ./multicast send ff15:: 1337 -a 5 -i 1000 -s 8 -t 4
07:29:28.606 (-04:00) : sent 8 byte(s) to [ff15::]:1337
07:29:29.608 (-04:00) : sent 8 byte(s) to [ff15::]:1337
07:29:30.610 (-04:00) : sent 8 byte(s) to [ff15::]:1337
07:29:31.610 (-04:00) : sent 8 byte(s) to [ff15::]:1337
07:29:32.612 (-04:00) : sent 8 byte(s) to [ff15::]:1337
```
