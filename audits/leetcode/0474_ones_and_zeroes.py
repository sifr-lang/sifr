from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 474: Ones And Zeroes
# Python version

from collections import defaultdict
from typing import List

def findMaxForm(strs: List[str], M: int, N: int) -> int:
    # Dynamic Programming
    dp = defaultdict(int)

    for s in strs:
        mCnt, nCnt = s.count("0"), s.count("1")
        for m in range(M, mCnt - 1, -1):
            for n in range(N, nCnt - 1, -1):
                dp[(m, n)] = max(
                    1 + dp[(m - mCnt, n - nCnt)],
                    dp[(m, n)])
    return dp[(M, N)]



def main():
    assert findMaxForm(["10", "0001", "111001", "1", "0"], 5, 3) == 4
    assert findMaxForm(["10", "0", "1"], 1, 1) == 2

if __name__ == "__main__":
    main()
