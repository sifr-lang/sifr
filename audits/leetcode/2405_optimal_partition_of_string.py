from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 2405: Optimal Partition Of String
# Python version

def partitionString(s: str) -> int:
    c=0
    res=set()
    for i in s:
        if i in res:
            c=c+1
            res=set()
        res.add(i)
    return c+1



def main():
    assert partitionString("abacbc") == 3
    assert partitionString("ssssss") == 6

if __name__ == "__main__":
    main()
