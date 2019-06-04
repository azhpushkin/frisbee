import socket
import sys
import yaml
import zmq
import threading
import uuid


ENV_UUID =  str(uuid.uuid4())

config = yaml.safe_load(open(sys.argv[1]))
local_conf = config[sys.argv[2]]

global_read_port = None
global_write_port = None


def start_zmq_queues():
    global global_read_port
    global global_write_port

    c = zmq.Context()

    read = c.socket(zmq.SUB)
    global_read_port = read.bind_to_random_port('tcp://127.0.0.1')
    read.subscribe('')  # Read all topics

    write = c.socket(zmq.PUB)
    global_write_port = write.bind_to_random_port('tcp://127.0.0.1')

    print(
        'Starting read queue on ',
        global_read_port,
        'and write queue on',
        global_write_port
    )

    while True:
        topic, data = read.recv_multipart()
        if topic.startswith(b'return'):
            topic = topic
        else:
            topic = b'messages:' + topic

        write.send_multipart([topic, data])

        print(f'[{topic}] {data}')


def start_client(sock):
    while True:
        data = sock.recv(1024)
        if not data:
            sock.close()
            return

        data = data.decode('ascii')
        if data == 'init':
            sock.send(f'{global_write_port}:{global_read_port}'.encode('ascii'))
            sock.close()
            return

        # sock.send(b"GOT" + str(global_write_port).encode('ascii'))
        # sock.send(b"GOT" + str(global_read_port).encode('ascii'))


def start_tcp_server():
    print('Starting TCP server on', local_conf['port'])
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.bind(('0.0.0.0', local_conf['port']))
    sock.listen()

    while True:
        client, _ = sock.accept()
        handler = threading.Thread(target=start_client, args=[client, ])
        handler.start()



if __name__ == '__main__':
    threading.Thread(target=start_zmq_queues).start()

    start_tcp_server()


