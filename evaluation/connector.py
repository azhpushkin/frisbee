import uuid
import zmq
import time

from .ast_def.expressions import *
from .passive_object import ExpPassiveObject


class ActorConnector:
    actor_id: str

    messages_socket: zmq.Socket
    return_socket: zmq.Socket
    write_socket: zmq.Socket

    def __init__(self, ):
        self.actor_id = str(uuid.uuid4())

        context = zmq.Context()
        self.messages_socket = context.socket(zmq.SUB)
        self.messages_socket.connect('tcp://127.0.0.1:5556')
        self.messages_socket.subscribe(f'messages:{self.actor_id}')
        # messages_socket.subscribe('')

        self.return_socket = context.socket(zmq.SUB)
        self.return_socket.connect('tcp://127.0.0.1:5556')
        self.return_socket.subscribe(f'return:{self.actor_id}')

        self.write_socket = context.socket(zmq.PUB)
        self.write_socket.connect('tcp://127.0.0.1:5557')

        time.sleep(0.2)  # ensure connection established

    def receive_message(self):
        topic, data = self.messages_socket.recv_multipart()
        data = eval(data)
        return data['name'], data['args'], data['return']

    def receive_return_value(self):
        topic, result = self.return_socket.recv_multipart()
        return eval(result.decode('ascii'))

    def return_result(self, return_actor, result):
        self.write_socket.send_multipart([
            'return:{}'.format(return_actor).encode('ascii'),
            str(result).encode('ascii')
        ])

    def send_message(self, actor_id, name, args, return_to: str = None):
        self.write_socket.send_multipart([
            actor_id.encode('ascii'),
            str({'name': name, 'args': args, 'return': return_to}).encode('ascii')
        ])


def send_initial_message(actor_id, name, args):
    context = zmq.Context()

    write_socket = context.socket(zmq.PUB)
    write_socket.connect('tcp://127.0.0.1:5557')
    time.sleep(0.2)
    write_socket.send_multipart([
        actor_id.encode('ascii'),
        str({'name': name, 'args': args, 'return': None}).encode('ascii')
    ])
