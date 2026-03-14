from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 322: Coin Change
# Python version

def coinChange(coins: List[int], amount: int) -> int:
    dp = [amount + 1] * (amount + 1)
    dp[0] = 0

    for a in range(1, amount + 1):
        for c in coins:
            if a - c >= 0:
                dp[a] = min(dp[a], 1 + dp[a - c])
    return dp[amount] if dp[amount] != amount + 1 else -1



def main():
    assert coinChange([1,2,5], 11) == 3
    assert coinChange([2], 3) == -1
    assert coinChange([1], 0) == 0

if __name__ == "__main__":
    main()
