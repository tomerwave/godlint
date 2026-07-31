from .gateway import settle

import os.path


def refund(order):
    return settle(order)
