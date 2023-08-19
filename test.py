import asyncio
from websockets.sync.client import connect


def main():
    with connect("ws://127.0.0.1:8282/updates") as websocket:
        websocket.send('{"Subscribe":[1]}')
        while True:
            message = websocket.recv()
            print(message)



if __name__ == '__main__':
    main()
