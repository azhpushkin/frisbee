import multiprocessing as mp
import socket
import typing

from . import ast_def
from . import global_conf
from .connector import ActorConnector
from .active_object import BaseActiveObject, BaseActiveObjectDeclaration


mp.allow_connection_pickling()


class TCPServerDeclaration(BaseActiveObjectDeclaration):
    def spawn(self, args):
        assert len(args) == 1, "One argument required for Socket"
        assert isinstance(args[0], ast_def.ExpInt), "Int required"

        tcp_server = TCPServerActiveObject(port=args[0].value)
        return tcp_server.start_and_return_proxy()


class TCPServerActiveObject(BaseActiveObject):
    def __init__(self, port: int):
        self.port = port
        self.sock = None
        self.connections = []

    def on_start(self):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.sock.bind(('localhost', self.port))
        self.sock.listen()

    def proceed_message(self, message_name, args):
        if message_name == 'accept':
            new_connection, addr = self.sock.accept()
            connection_actor = TCPConnectionActiveObject(socket=new_connection)
            proxy = connection_actor.start_and_return_proxy()
            self.connections.append(proxy)
            return proxy
        else:
            raise Exception()


class TCPConnectionActiveObject(BaseActiveObject):
    def __init__(self, socket):
        self.sock = socket

    def proceed_message(self, message_name: str, args):
        if message_name == 'get':
            data = self.sock.recv(1024)
            if not data:
                return ast_def.ExpVoid()
            else:
                return ast_def.ExpString(value=data.decode('ascii').strip())

        elif message_name == 'send':
            data = str(args).encode('ascii') + b'\n'
            self.sock.send(data)
        else:
            raise Exception()








BUILTIN_TYPES = {
    'sockets': {
        'TCPServer': TCPServerDeclaration()
    }
}
