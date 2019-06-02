from dataclasses import dataclass
import typing



@dataclass
class BaseObjectDeclList:
    def get_declarations(self) -> typing.List[BaseObjectDecl]:
        return NotImplemented


@dataclass
class ObjectDeclList(BaseObjectDeclList):
    head: BaseObjectDecl
    tail: BaseObjectDeclList

    def get_declarations(self):
        return [self.head, ] + self.tail.get_declarations()


@dataclass
class OEmpty(BaseObjectDeclList):
    def get_declarations(self):
        return []





@dataclass
class Program:
    imports: BaseImportDeclList
    objects: BaseObjectDeclList