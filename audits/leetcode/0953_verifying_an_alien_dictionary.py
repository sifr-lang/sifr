from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 953: Verifying An Alien Dictionary
# Python version

def isAlienSorted(words: List[str], order: str) -> bool:
    # first differing char
    # if word A is prefix of word B, word B must be AFTER word A
    orderInd = { c : i for i, c in enumerate(order)}
    
    for i in range(len(words) - 1):
        w1, w2 = words[i], words[i + 1]
        
        for j in range(len(w1)):
            if j == len(w2):
                return False
            
            if w1[j] != w2[j]:
                if orderInd[w2[j]] < orderInd[w1[j]]:
                    return False
                break
    return True



def main():
    assert isAlienSorted(['hello', 'leetcode'], 'hlabcdefgijkmnopqrstuvwxyz') == True
    assert isAlienSorted(['word', 'world', 'row'], 'worldabcefghijkmnpqstuvxyz') == False
    assert isAlienSorted(['apple', 'app'], 'abcdefghijklmnopqrstuvwxyz') == False

if __name__ == "__main__":
    main()
