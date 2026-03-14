from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 187: Repeated Dna Sequences
# Python version

def findRepeatedDnaSequences(s: str) -> list[str]:
    result = set()
    previous_sequences = set()
    for i in range(len(s) - 9):
        current = s[i:i+10]
        if current in previous_sequences:
            result.add(current)
        previous_sequences.add(current)
    return list(result)



def main():
    assert sorted(findRepeatedDnaSequences('AAAAACCCCCAAAAACCCCCCAAAAAGGGTTT')) == ['AAAAACCCCC', 'CCCCCAAAAA']
    assert sorted(findRepeatedDnaSequences('AAAAAAAAAAAAA')) == ['AAAAAAAAAA']

if __name__ == "__main__":
    main()
