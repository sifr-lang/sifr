from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 205: Isomorphic Strings
# Python version

def isIsomorphic(s: str, t: str) -> bool:
    mapST, mapTS = {}, {}

    for c1, c2 in zip(s, t):
        if (c1 in mapST and mapST[c1] != c2) or (c2 in mapTS and mapTS[c2] != c1):
            return False
        mapST[c1] = c2
        mapTS[c2] = c1

    return True


def main():
    assert isIsomorphic("egg", "add") == True
    assert isIsomorphic("foo", "bar") == False

if __name__ == "__main__":
    main()
