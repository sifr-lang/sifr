from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1849: Splitting A String Into Descending Consecutive Values
# Python version

def splitString(s: str) -> bool:
    
    def dfs(index, prev):
        if index == len(s):
            return True
    
        for j in range(index, len(s)):
            val = int(s[index:j+1])
            if val + 1 == prev and dfs(j+1, val):
                return True
        return False

    for i in range(len(s) - 1):
        val = int(s[:i + 1])
        if dfs(i+1, val): return True
            
    return False



def main():
    assert splitString("1234") == False
    assert splitString("050043") == True

if __name__ == "__main__":
    main()
