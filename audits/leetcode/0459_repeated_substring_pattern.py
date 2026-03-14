from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 459: Repeated Substring Pattern
# Python version

def repeatedSubstringPattern(s: str) -> bool:
    return s in (s + s)[1:-1]
    



def main():
    assert repeatedSubstringPattern("abab") == True
    assert repeatedSubstringPattern("aba") == False

if __name__ == "__main__":
    main()
