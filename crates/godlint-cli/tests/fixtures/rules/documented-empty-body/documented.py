from abc import abstractmethod


class Base:
    @abstractmethod
    def accepted_abstract(self):
        pass

    def accepted_documented(self):
        """Intentionally does nothing."""


def reported():
    pass
