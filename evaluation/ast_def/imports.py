from __future__ import annotations

import typing
from dataclasses import dataclass

__all__ = [
    'BaseImportDeclList',
    'ImportDeclList',
    'ImportDeclListEmpty',

    'BaseImportIdentList',
    'ImportIdentList',
    'ImportIdentListEmpty',
]


@dataclass
class BaseImportDeclList:
    def get_imports(self) -> typing.Dict[str, typing.List[str]]:
        return NotImplemented


@dataclass
class ImportDeclList(BaseImportDeclList):
    module: str
    typenames: BaseImportIdentList
    tail: BaseImportDeclList

    def get_imports(self):
        return {self.module: self.typenames.get_names(), **self.tail.get_imports()}


@dataclass
class ImportDeclListEmpty(BaseImportDeclList):
    def get_imports(self):
        return {}


@dataclass
class BaseImportIdentList:
    def get_names(self):
        return NotImplemented


@dataclass
class ImportIdentList(BaseImportIdentList):
    typename: str
    tail: BaseImportIdentList

    def get_names(self):
        return [self.typename, ] + self.tail.get_names()


@dataclass
class ImportIdentListEmpty(BaseImportIdentList):
    def get_names(self):
        return []
