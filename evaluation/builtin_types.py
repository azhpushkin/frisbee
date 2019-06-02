import multiprocessing as mp
import socket
import typing

from . import ast_def
from . import global_conf
from .connector import ActorConnector


class BuiltinPassiveDecl:
    def create(self, args: typing.List[ast_def.BaseExp]):
        return NotImplemented


class BuiltinActiveDecl:
    def spawn(self, args: typing.List[ast_def.BaseExp]):
        return NotImplemented


def start_socket(port, event: mp.Event, assigned_id: mp.Array):
    print(123)
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(('localhost', port))
    print(2)
    sock.listen()

    global_conf.local_connector = ActorConnector()
    assigned_id.value = global_conf.local_connector.actor_id.encode('ascii')

    event.set()
    accepted = None
    print(3, accepted)

    while True:
        print(11, global_conf.local_connector.actor_id)
        message_name, args, return_address = eval(global_conf.local_connector.receive_message())

        if not accepted:
            print('waiting')
            accepted, _ = sock.accept()
            print('got')

        print(message_name, args, return_address)
        if message_name == 'get':
            data = accepted.recv(1024).decode('ascii')
            print('Received data', data)
            result = ast_def.ExpString(value=data)
        elif message_name == 'send':
            data = str(args)
            print('Sending ', data)
            accepted.send(data.encode('ascii'))
        else:
            raise ValueError('Unknown value ' + str(message_name))

        if return_address:
            global_conf.local_connector.return_result(return_address, result)


class SocketActiveDeclaration(BuiltinActiveDecl):
    def spawn(self, args):
        port = getattr(args[0], 'value', None)  # ExpInt or ExpVoid
        spawned_event = mp.Event()
        assigned_id = mp.Array('c', 64)

        proc = mp.Process(target=start_socket, args=(port, spawned_event, assigned_id))
        proc.start()
        spawned_event.wait()

        return ast_def.ActiveProxy(actor_id=assigned_id.value.decode('ascii'))





BUILTIN_TYPES = {
    'socket': {
        'SocketServer': SocketActiveDeclaration()
    }
}
