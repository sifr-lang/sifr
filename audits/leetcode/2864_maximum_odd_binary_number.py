from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 2864: Maximum Odd Binary Number
# Python version

def maximumOddBinaryNumber(s: str) -> str:
    count = 0
    for c in s:
        if c == "1":
            count += 1
    
    return (count - 1) * "1" + (len(s) - count) * "0" + "1"

# Traverse and swap indices

def maximumOddBinaryNumber(s: str) -> str:
    s = [c for c in s]
    left = 0

    for i in range(len(s)):
        if s[i] == "1":
            s[i], s[left] = s[left], s[i]
            left += 1
    s[left - 1], s[len(s) - 1] = s[len(s) - 1], s[left - 1]
    return "".join(s)



def main():
    assert maximumOddBinaryNumber("010") == '001'
    assert maximumOddBinaryNumber("0101") == '1001'

if __name__ == "__main__":
    main()
