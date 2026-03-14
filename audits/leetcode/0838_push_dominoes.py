from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 838: Push Dominoes
# Python version

def pushDominoes(dominoes: str) -> str:
    dom = list(dominoes)
    q = collections.deque()
    for i, d in enumerate(dom):
        if d != '.':
            q.append((i, d))
    
    while q:
        i, d = q.popleft()

        if d == 'L' and i > 0 and dom[i - 1] == '.':
            q.append((i - 1, 'L'))
            dom[i - 1] = 'L'
        elif d == 'R':
            if i + 1 < len(dom) and dom[i + 1] == '.':
                if i + 2 < len(dom) and dom[i + 2] == 'L':
                    q.popleft()
                else:
                    q.append((i + 1, 'R'))
                    dom[i + 1] = 'R'

    return ''.join(dom)



def main():
    assert pushDominoes('RR.L') == 'RR.L'
    assert pushDominoes('.L.R...LR..L..') == 'LL.RR.LLRRLL..'

if __name__ == "__main__":
    main()
