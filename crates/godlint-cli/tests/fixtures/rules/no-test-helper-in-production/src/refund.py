from ..tests.helpers import fake_gateway


def refund(order):
    return fake_gateway.settle(order)
