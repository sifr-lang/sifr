from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 6: Zigzag Conversion
# Python version

def convert(s: str, numRows: int) -> str:
    if numRows == 1 or numRows >= len(s):
        return s

    res = [""] * numRows

    index = 0
    step = 1
    for c in s:
        res[index] += c
        if index == 0:
            step = 1
        elif index == numRows - 1:
            step = -1
        index += step

    return "".join(res)


def main():
    assert convert("PAYPALISHIRING", 3) == 'PAHNAPLSIIGYIR'
    assert convert("A", 1) == 'A'

if __name__ == "__main__":
    main()
