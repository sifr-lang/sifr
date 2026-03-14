from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 274: H Index
# Python version

def hIndex(citations: List[int]) -> int:
    length = len(citations)
    citations.sort()
    for i in range(length):
        if citations[i] >= length - i:
            return length - i
    return 0



def main():
    assert hIndex([3,0,6,1,5]) == 3

if __name__ == "__main__":
    main()
