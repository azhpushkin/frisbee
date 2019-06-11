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
read_socket = None
global_write_port = None
write_socket = None

main_actor: str

env_connections = {

}

actors_mapping = {

}
other_mains = {

}
local_actors = set()


def send_to_actor(actor_id, topic, data):
    if actor_id in local_actors:
        print('LOCAL')
        write_socket.send_multipart([topic.encode('ascii'), data])
    elif actor_id in actors_mapping:
        print('NOT LOCAL', topic, data)
        env_uuid = actors_mapping[actor_id]
        sock = env_connections[env_uuid]
        print(1, f'{topic}##{data.decode("ascii")}')
        sock.send(f'{topic}##{data.decode("ascii")}'.encode('ascii'))
        print(2)
    else:
        print(4, actors_mapping.keys())
        for env_uuid in set(env_connections.keys()):
            print(33, env_uuid)
            env_connections[env_uuid].send(f'{topic}##{data.decode("ascii")}'.encode('ascii'))


def start_zmq_queues():
    global global_read_port
    global global_write_port
    global main_actor
    global local_actors
    global write_socket
    global read_socket

    c = zmq.Context()

    read_socket = c.socket(zmq.SUB)
    global_read_port = read_socket.bind_to_random_port('tcp://127.0.0.1')
    read_socket.subscribe('')  # Read all topics

    write_socket = c.socket(zmq.PUB)
    global_write_port = write_socket.bind_to_random_port('tcp://127.0.0.1')

    print(
        'Starting read queue on ',
        global_read_port,
        'and write queue on',
        global_write_port
    )

    while True:
        topic, data = read_socket.recv_multipart()
        typeof, actor_id = topic.decode('ascii').split(':')
        print('[{:7}][{}] {}'.format(typeof, actor_id, data))

        if typeof == 'return':
            send_to_actor(actor_id, f'return:{actor_id}', data)
        elif typeof == 'message':
            send_to_actor(actor_id, f'messages:{actor_id}', data)
        elif typeof == 'main':
            main_actor = actor_id
        elif typeof == 'create':
            local_actors.add(actor_id)


def start_client(sock):
    while True:
        data = sock.recv(1024)
        if not data:
            sock.close()
            return

        data = data.decode('ascii')
        if data == 'init':

            print(other_mains)
            print('#### NEW PROGRAM CONNECTING ####')
            sock.send(f'{global_write_port}:{global_read_port}'.encode('ascii'))
            sock.recv(512)  # ACK
            sock.send(str(other_mains).encode('ascii'))
            return
        elif data.startswith('remote:'):
            new_env_uuid = data.split(':')[1]
            print('NEW ENV UUID', new_env_uuid)
            sock.send(f'{ENV_UUID}:{main_actor}'.encode('ascii'))
            env_connections[new_env_uuid] = sock
            threading.Thread(target=start_receiving, args=[sock, ]).start()
            return


def start_receiving(sock):
    print('GOGO')
    while True:
        data = sock.recv(1024)
        if not data:
            return

        print(3, data)
        topic, data = data.decode('ascii').split('##')
        _, actor_id = topic.split(':')
        if actor_id not in local_actors:
            continue

        send_to_actor(actor_id, topic, data.encode('ascii'))


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
        env_uuid, main_uuid = data.split(':')
        print("GOT MAIN FROM {}: {}".format(c, main_uuid))

        env_connections[env_uuid] = s
        actors_mapping[main_uuid] = env_uuid

        other_mains[c] = main_uuid

    threading.Thread(target=start_zmq_queues).start()

    start_tcp_server()


