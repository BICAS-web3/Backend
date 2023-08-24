import asyncio
from websockets.sync.client import connect


def main():
    with connect("ws://127.0.0.1:8585/api/updates") as websocket:
        print("Connected")
        websocket.send('{"type":"Subscribe","payload":["CoinFlip"]}')
        while True:
            message = websocket.recv()
            print(message)


if __name__ == '__main__':
    main()
