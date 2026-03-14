from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 241: Different Ways To Add Parentheses
# Python version

def diffWaysToCompute(s: str) -> List[int]:
    def f(s):
        res = []
        for i, c in enumerate(s):
            if c in '+-*':
                for l in f(s[:i]):
                    for r in f(s[i + 1:]):
                        res.append(eval(f'{l}{c}{r}'))
        
        return res or [int(s)]
    return f(s)



def main():
    assert diffWaysToCompute("2-1-1") == [2, 0]

if __name__ == "__main__":
    main()
