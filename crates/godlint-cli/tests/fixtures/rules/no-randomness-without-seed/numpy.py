import numpy as np


def test_matrix_is_stable():
    matrix = np.random.rand(3, 3)
    assert matrix.shape == (3, 3)
