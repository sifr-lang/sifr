from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 2390: Removing Stars From A String
# Python version

def removeStars(s) :
    res = []
    for c in s :
        if res and c == '*':
            res.pop()
        else:
            res.append(c)
    return ''.join(res)



def main():
    assert removeStars("leet**cod*e") == 'lecoe'

if __name__ == "__main__":
    main()
