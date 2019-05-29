import zmq
import time


class ActorConnector:
    actor_uuid: str

    messages_socket: zmq.Socket
    return_socket: zmq.Socket
    write_socket: zmq.Socket

    def __init__(self, actor_uuid: str):
        self.actor_uuid = actor_uuid

        context = zmq.Context()
        self.messages_socket = context.socket(zmq.SUB)
        self.messages_socket.connect('tcp://127.0.0.1:5556')
        self.messages_socket.subscribe(f'messages:{actor_uuid}')
        # messages_socket.subscribe('')

        self.return_socket = context.socket(zmq.SUB)
        self.return_socket.connect('tcp://127.0.0.1:5556')
        self.return_socket.subscribe(f'return:{actor_uuid}')

        self.write_socket = context.socket(zmq.PUB)
        self.write_socket.connect('tcp://127.0.0.1:5557')

        time.sleep(0.2)  # ensure connection established

    def receive_message(self):
        topic, data = self.messages_socket.recv_multipart()
        return data.decode('ascii')
        # return data['name'], data['args'], data['return']

    def receive_return_value(self):
        topic, result = self.return_socket.recv_multipart()
        return result.decode('ascii')

    def return_result(self, return_actor, result):
        self.write_socket.send_multipart([
            'return:{}'.format(return_actor).encode('ascii'),
            str(result).encode('ascii')
        ])

    def send_message(self, actor_uuid, name, args, return_to):
        return_uuid = return_to.actor_uuid if return_to else None
        self.write_socket.send_multipart([
            actor_uuid.encode('ascii'),
            str({'name': name, 'args': args, 'return': return_uuid}).encode('ascii')
        ])


def send_initial_message(actor_uuid, name, args):
    context = zmq.Context()

    write_socket = context.socket(zmq.PUB)
    write_socket.connect('tcp://127.0.0.1:5557')
    time.sleep(0.2)
    write_socket.send_multipart([
        actor_uuid.encode('ascii'),
        str({'name': name, 'args': args, 'return': None}).encode('ascii')
    ])
