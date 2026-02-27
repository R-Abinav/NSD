from scapy.all import rdpcap, TCP, IP

def analyze(file):
    packets = rdpcap(file)
    connections = {}

    for pkt in packets:
        if IP not in pkt or TCP not in pkt:
            continue

        src = pkt[IP].src
        dst = pkt[IP].dst
        sport = pkt[TCP].sport
        dport = pkt[TCP].dport
        flags = pkt[TCP].flags

        #normalize key so both directions map to same connection
        key = tuple(sorted([(src, sport), (dst, dport)]))

        if key not in connections:
            connections[key] = {'syn': 0, 'synack': 0, 'ack': 0}

        if flags & 0x02 and flags & 0x10:
            connections[key]['synack'] += 1
        elif flags & 0x02:
            connections[key]['syn'] += 1
        elif flags & 0x10:
            connections[key]['ack'] += 1

    print(f"total connections found: {len(connections)}\n")

    completed = 0
    failed = 0

    for key, counts in connections.items():
        (a_ip, a_port), (b_ip, b_port) = key
        syn = counts['syn']
        synack = counts['synack']
        ack = counts['ack']

        if syn >= 1 and synack >= 1 and ack >= 1:
            status = "completed"
            completed += 1
        else:
            status = "incomplete/failed"
            failed += 1

        print(f"{a_ip}:{a_port} <-> {b_ip}:{b_port} | syn={syn} synack={synack} ack={ack} | {status}")

    print(f"\ncompleted: {completed} | failed/incomplete: {failed}")

analyze("capture.pcap")