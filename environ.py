import socket
import sys
import yaml
import zmq
import threading
import uuid


config = yaml.safe_load(open(sys.argv[1]))
local_conf = config[sys.argv[2]]
ENV_NAME = sys.argv[2]


c = zmq.Context()

read_socket = c.socket(zmq.SUB)
global_read_port = read_socket.bind_to_random_port('tcp://127.0.0.1')
read_socket.subscribe('')  # Read all topics

write_socket = c.socket(zmq.PUB)
global_write_port = write_socket.bind_to_random_port('tcp://127.0.0.1')

main_actor: str = None

env_connections = {

}

other_mains = {

}
local_actors = set()


def establish_connection_to_env(env_name, *, with_main=True):
    print(">> Connecting to {}... ".format(c))
    con_conf = config[env_name]
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((con_conf['ip'], int(con_conf['port'])))
    s.send(f'remote:{ENV_NAME}'.encode('ascii'))

    data = s.recv(1024).decode('ascii')
    threading.Thread(target=start_receiving, args=[s, env_name]).start()
    main_uuid = data.split(':')

    env_connections[env_name] = s
    if with_main:
        other_mains[env_name] = main_uuid[0]
        print('    Done! [main: {}]'.format(main_uuid))
    else:
        print('    Done!')


def send_to_actor(actor_id, env_name, msg_type, data):
    if msg_type == 'return':
        topic = 'return:{}:{}'.format(actor_id, env_name)
    elif msg_type == 'message':
        topic = 'message:{}:{}'.format(actor_id, env_name)
    else:
        raise ValueError('Unknown type {}'.format(msg_type))

    if env_name == ENV_NAME:
        write_socket.send_multipart([topic.encode('ascii'), data])
    elif env_name in env_connections:
        sock = env_connections[env_name]
        sock.send(f'{topic}#<>#{data.decode("ascii")}'.encode('ascii'))
    else:
        establish_connection_to_env(env_name, with_main=False)
        sock = env_connections[env_name]
        sock.send(f'{topic}#<>#{data.decode("ascii")}'.encode('ascii'))


def proceed_message(topic, data):
    global main_actor
    global local_actors

    if not isinstance(topic, str):
        topic = topic.decode('ascii')

    if isinstance(data, str):
        data = data.encode('ascii')

    topic_args = topic.split(':')

    if topic_args[0] == 'return':
        send_to_actor(topic_args[1], topic_args[2], 'return', data)
        print('[RETURN][env:{:6}][{}]'.format(topic_args[2], topic_args[1]))
        print('  Args:', data)

    elif topic_args[0] == 'message':
        send_to_actor(topic_args[1], topic_args[2], 'message', data)
        print('[NEWMSG][env:{:6}][{}]'.format(topic_args[2], topic_args[1]))
        print('  Args:', data)

    elif topic_args[0] == 'main':
        print('MAIN connected: [{}]'.format(topic_args[1]))
        main_actor = topic_args[1]

    elif topic_args[0] == 'create':
        if main_actor is not None:
            print('New actor created: [{}]'.format(topic_args[1]))
        local_actors.add(topic_args[1])



def start_zmq_queues():
    print(
        'Starting read queue on',
        global_read_port,
        'and write queue on',
        global_write_port
    )

    while True:
        topic, data = read_socket.recv_multipart()
        proceed_message(topic, data)



def start_client(sock):
    while True:
        data = sock.recv(1024)
        if not data:
            sock.close()
            return

        data = data.decode('ascii')
        if data == 'init':

            print('Program connected!')
            sock.send(f'{ENV_NAME}:{global_write_port}:{global_read_port}'.encode('ascii'))
            sock.recv(512)  # ACK
            sock.send(str(other_mains).encode('ascii'))
            sock.close()
            return

        elif data.startswith('remote:'):
            new_env_name = data.split(':')[1]
            sock.send(main_actor.encode('ascii'))
            env_connections[new_env_name] = sock
            threading.Thread(target=start_receiving, args=[sock, new_env_name]).start()
            return


def start_receiving(sock, env_name):
    print('>> New env connected: {}'.format(env_name))

    while True:
        data = sock.recv(1024)
        if not data:
            return

        topic, data = data.decode('ascii').split('#<>#')
        proceed_message(topic, data)


def start_tcp_server():
    print('Starting TCP server on', local_conf['port'])
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(('0.0.0.0', local_conf['port']))
    sock.listen()

    print('>> All done, awaiting program to connect... ')

    while True:
        client, _ = sock.accept()
        handler = threading.Thread(target=start_client, args=[client, ])
        handler.start()


if __name__ == '__main__':
    print('LAUNCHING ENV', ENV_NAME)

    connections = local_conf.get('connections', [])
    if connections:
        print("Connecting to linked environments")
        for c in connections:
            establish_connection_to_env(c)

    print("Starting local queues")

    threading.Thread(target=start_zmq_queues).start()

    start_tcp_server()


