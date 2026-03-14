from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 127: Word Ladder
# Python version

def ladderLength(beginWord: str, endWord: str, wordList: List[str]) -> int:
    if endWord not in wordList:
        return 0

    nei = collections.defaultdict(list)
    wordList.append(beginWord)
    for word in wordList:
        for j in range(len(word)):
            pattern = word[:j] + "*" + word[j + 1 :]
            nei[pattern].append(word)

    visit = set([beginWord])
    q = deque([beginWord])
    res = 1
    while q:
        for i in range(len(q)):
            word = q.popleft()
            if word == endWord:
                return res
            for j in range(len(word)):
                pattern = word[:j] + "*" + word[j + 1 :]
                for neiWord in nei[pattern]:
                    if neiWord not in visit:
                        visit.add(neiWord)
                        q.append(neiWord)
        res += 1
    return 0



def main():
    assert ladderLength('hit', 'cog', ['hot', 'dot', 'dog', 'lot', 'log', 'cog']) == 5
    assert ladderLength('hit', 'cog', ['hot', 'dot', 'dog', 'lot', 'log']) == 0

if __name__ == "__main__":
    main()
