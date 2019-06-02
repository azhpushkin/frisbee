from __future__ import annotations

from dataclasses import dataclass


@dataclass
class BaseType:
    pass


@dataclass
class TypeAnonymous(BaseType): pass


@dataclass
class TypeMaybe(BaseType):
    type: BaseType


@dataclass
class TypeArray(BaseType):
    type: BaseType


@dataclass
class TypeInt(BaseType): pass


@dataclass
class TypeVoid(BaseType): pass


@dataclass
class TypeBool(BaseType): pass


@dataclass
class TypeString(BaseType): pass


@dataclass
class TypeIdent(BaseType):
    name: str
