from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 621: Task Scheduler
# Python version

def leastInterval(tasks: List[str], n: int) -> int:
    count = Counter(tasks)
    maxHeap = [-cnt for cnt in count.values()]
    heapq.heapify(maxHeap)

    time = 0
    q = deque()  # pairs of [-cnt, idleTime]
    while maxHeap or q:
        time += 1

        if not maxHeap:
            time = q[0][1]
        else:
            cnt = 1 + heapq.heappop(maxHeap)
            if cnt:
                q.append([cnt, time + n])
        if q and q[0][1] == time:
            heapq.heappush(maxHeap, q.popleft()[0])
    return time


# Greedy algorithm

def leastInterval(tasks: List[str], n: int) -> int:
    counter = collections.Counter(tasks)
    max_count = max(counter.values())
    min_time = (max_count - 1) * (n + 1) + \
                sum(map(lambda count: count == max_count, counter.values()))

    return max(min_time, len(tasks))


def main():
    assert leastInterval(['A', 'A', 'A', 'B', 'B', 'B'], 2) == 8
    assert leastInterval(['A', 'C', 'A', 'B', 'D', 'B'], 1) == 6
    assert leastInterval(['A', 'A', 'A', 'B', 'B', 'B'], 3) == 10

if __name__ == "__main__":
    main()
