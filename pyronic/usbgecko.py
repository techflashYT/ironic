#!/usr/bin/env python3
import argparse
import os
import select
import socket
import sys
import time

# Ctrl-], as in telnet
ESCAPE = 0x1d


def connect(host, port, wait):
    printed = False
    while True:
        try:
            sock = socket.create_connection((host, port))
            sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
            return sock
        except OSError as e:
            if not wait:
                print(f"usbgecko: can't connect to {host}:{port}: {e}",
                      file=sys.stderr)
                print("usbgecko: is ironic running with --usbgecko? "
                      "(-w retries until it is)", file=sys.stderr)
                sys.exit(1)
            if not printed:
                print(f"usbgecko: waiting for {host}:{port} ...",
                      file=sys.stderr)
                printed = True
            time.sleep(0.5)


def pump(sock, escape_active):
    """ Shuttle bytes between stdin/stdout and the gecko socket.
    Returns when the connection drops or (if escape_active) on Ctrl-]. """
    stdin_fd = sys.stdin.fileno()
    stdin_open = True
    while True:
        fds = [sock] + ([stdin_fd] if stdin_open else [])
        readable, _, _ = select.select(fds, [], [])
        if sock in readable:
            data = sock.recv(4096)
            if not data:
                return
            sys.stdout.buffer.write(data)
            sys.stdout.buffer.flush()
        if stdin_fd in readable:
            data = os.read(stdin_fd, 4096)
            if not data:
                # EOF on piped input: keep printing guest output
                stdin_open = False
                continue
            if escape_active and ESCAPE in data:
                sock.sendall(data[:data.index(ESCAPE)])
                return
            sock.sendall(data)


def main():
    parser = argparse.ArgumentParser(
        description="Terminal client for ironic's emulated USB Gecko")
    parser.add_argument("--host", default="127.0.0.1",
                        help="host to connect to (default: 127.0.0.1)")
    parser.add_argument("-p", "--port", type=int, default=55021,
                        help="TCP port of the gecko server (default: 55021)")
    parser.add_argument("-w", "--wait", action="store_true",
                        help="retry until the gecko server is reachable")
    args = parser.parse_args()

    sock = connect(args.host, args.port, args.wait)

    if sys.stdin.isatty():
        import termios
        import tty
        print(f"usbgecko: connected to {args.host}:{args.port}, "
              "escape character is Ctrl-]", file=sys.stderr)
        saved = termios.tcgetattr(sys.stdin.fileno())
        tty.setraw(sys.stdin.fileno())
        mode = termios.tcgetattr(sys.stdin.fileno())
        mode[1] |= termios.OPOST | termios.ONLCR  # oflags prevent staircasing
        termios.tcsetattr(sys.stdin.fileno(), termios.TCSADRAIN, mode)
        try:
            pump(sock, escape_active=True)
        finally:
            termios.tcsetattr(sys.stdin.fileno(), termios.TCSADRAIN, saved)
            print("\nusbgecko: detached", file=sys.stderr)
    else:
        try:
            pump(sock, escape_active=False)
        except KeyboardInterrupt:
            pass
        print("usbgecko: connection closed", file=sys.stderr)


if __name__ == "__main__":
    main()
