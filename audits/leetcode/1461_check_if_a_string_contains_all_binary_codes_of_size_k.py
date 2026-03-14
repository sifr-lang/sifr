from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1461: Check If A String Contains All Binary Codes Of Size K
# Python version

def hasAllCodes(s: str, k: int) -> bool:
    return len(set(s[i : i + k] for i in range(len(s) - k + 1))) == 2**k



def main():
    assert hasAllCodes('00110110', 2) == True
    assert hasAllCodes('0110', 1) == True
    assert hasAllCodes('0110', 2) == False

if __name__ == "__main__":
    main()
