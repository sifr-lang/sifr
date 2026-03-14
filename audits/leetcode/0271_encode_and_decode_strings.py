from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 271: Encode And Decode Strings
# Python version

def encode(strs):
    res = ""
    for s in strs:
        res += str(len(s)) + "#" + s
    return res


def decode(s):
    res = []
    i = 0
    
    while i < len(s):
        j = i
        while s[j] != '#':
            j += 1
        length = int(s[i:j])
        i = j + 1
        j = i + length
        res.append(s[i:j])
        i = j
        
    return res



def main():
    assert encode(['Hello', 'World']) == '5#Hello5#World'
    assert encode(['']) == '0#'

if __name__ == "__main__":
    main()
