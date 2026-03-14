from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 151: Reverse Words In A String
# Python version

def reverseWords(s: str) -> str:
    # Remove leading and trailing spaces
    s = s.strip()
    
    # Split the string into words
    words = s.split()
    
    # Reverse the order of words
    words = words[::-1]
    
    # Join the words with a single space
    reversed_str = ' '.join(words)
    
    return reversed_str



def main():
    assert reverseWords("the sky is blue") == 'blue is sky the'

if __name__ == "__main__":
    main()
