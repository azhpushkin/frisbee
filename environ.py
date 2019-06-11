import socket
import sys
import yaml
import zmq
import threading
import uuid


ENV_UUID = str(uuid.uuid4())

config = yaml.safe_load(open(sys.argv[1]))
local_conf = config[sys.argv[2]]

global_read_port = None
global_write_port = None

main_actor: str
local_actors = []

env_connections = {

}

actors_mapping = {

}


def start_zmq_queues():
    global global_read_port
    global global_write_port
    global main_actor
    global local_actors

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
        print(topic)
        typeof, actor_id = topic.decode('ascii').split(':')
        print('[{:7}][{}] {}'.format(typeof, actor_id, data))

        if typeof == 'return':
            res_topic = f'return:{actor_id}'
        elif typeof == 'message':
            res_topic = f'messages:{actor_id}'
        elif typeof == 'main':
            main_actor = actor_id
            continue
        elif typeof == 'create':
            local_actors.append(actor_id)
            continue
        else:
            continue

        write.send_multipart([res_topic.encode('ascii'), data])


def start_client(sock):
    while True:
        data = sock.recv(1024)
        if not data:
            sock.close()
            return

        data = data.decode('ascii')
        if data == 'init':
            print('#### NEW PROGRAM CONNECTING ####')
            sock.send(f'{global_write_port}:{global_read_port}'.encode('ascii'))
            sock.close()
            return
        elif data.startswith('remote:'):
            new_env_uuid = data.split(':')[1]
            print('NEW ENV UUID', new_env_uuid)
            sock.send(f'{ENV_UUID}:{main_actor}'.encode('ascii'))
            sock.close()
            return


def start_tcp_server():
    print('Starting TCP server on', local_conf['port'])
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(('0.0.0.0', local_conf['port']))
    sock.listen()

    while True:
        client, _ = sock.accept()
        handler = threading.Thread(target=start_client, args=[client, ])
        handler.start()


if __name__ == '__main__':
    print('LAUNCHING ENV', ENV_UUID)
    connections = local_conf.get('connections', [])
    for c in connections:
        con_conf = config[c]
        print(con_conf)
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.connect((con_conf['ip'], int(con_conf['port'])))
        s.send(f'remote:{ENV_UUID}'.encode('ascii'))

        data = s.recv(1024).decode('ascii')
        self_uuid, main_uuid = data.split(':')
        print("GOT MAIN FROM {}: {}".format(self_uuid, main_uuid))
        s.close()

    threading.Thread(target=start_zmq_queues).start()

    start_tcp_server()


