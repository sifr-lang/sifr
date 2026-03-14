from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 67: Add Binary
# Python version

def addBinary(a: str, b: str) -> str:
    res = ""
    carry = 0

    a, b = a[::-1], b[::-1]
    for i in range(max(len(a), len(b))):
        bitA = ord(a[i]) - ord('0') if i < len(a) else 0
        bitB = ord(b[i]) - ord('0') if i < len(b) else 0

        total = bitA + bitB + carry
        char = str(total % 2)
        res = char + res
        carry = total // 2

    if carry:
        res = "1" + res

    return res



def main():
    assert addBinary("11", "1") == '100'
    assert addBinary("1010", "1011") == '10101'

if __name__ == "__main__":
    main()
