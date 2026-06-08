import sys, json, socket, base64, os, struct, urllib.request

def get_page_ws():
    data = json.load(urllib.request.urlopen("http://127.0.0.1:9222/json"))
    for t in data:
        if t.get("type") == "page":
            return t["webSocketDebuggerUrl"]
    raise SystemExit("no page target")

def ws_connect(url):
    # ws://127.0.0.1:9222/devtools/page/XXXX
    rest = url[len("ws://"):]
    hostport, path = rest.split("/", 1)
    host, port = hostport.split(":")
    s = socket.create_connection((host, int(port)))
    key = base64.b64encode(os.urandom(16)).decode()
    req = (
        f"GET /{path} HTTP/1.1\r\nHost: {hostport}\r\n"
        "Upgrade: websocket\r\nConnection: Upgrade\r\n"
        f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
    )
    s.sendall(req.encode())
    buf = b""
    while b"\r\n\r\n" not in buf:
        buf += s.recv(1)
    return s

def ws_send(s, data):
    payload = data.encode()
    header = bytearray([0x81])
    n = len(payload)
    mask = os.urandom(4)
    if n < 126:
        header.append(0x80 | n)
    elif n < 65536:
        header.append(0x80 | 126); header += struct.pack(">H", n)
    else:
        header.append(0x80 | 127); header += struct.pack(">Q", n)
    header += mask
    masked = bytes(b ^ mask[i % 4] for i, b in enumerate(payload))
    s.sendall(bytes(header) + masked)

def ws_recv(s):
    b1 = s.recv(1)
    if not b1: return None
    b2 = s.recv(1)[0]
    n = b2 & 0x7f
    if n == 126: n = struct.unpack(">H", s.recv(2))[0]
    elif n == 127: n = struct.unpack(">Q", s.recv(8))[0]
    data = b""
    while len(data) < n:
        data += s.recv(n - len(data))
    return data.decode(errors="replace")

expr = sys.argv[1] if len(sys.argv) > 1 else "1"
s = ws_connect(get_page_ws())
ws_send(s, json.dumps({
    "id": 1, "method": "Runtime.evaluate",
    "params": {"expression": expr, "returnByValue": True, "awaitPromise": True}
}))
for _ in range(50):
    msg = ws_recv(s)
    if not msg: break
    o = json.loads(msg)
    if o.get("id") == 1:
        r = o.get("result", {})
        if "exceptionDetails" in r:
            print("EXCEPTION:", json.dumps(r["exceptionDetails"], ensure_ascii=False))
        else:
            print(json.dumps(r.get("result", {}).get("value"), ensure_ascii=False))
        break
s.close()
