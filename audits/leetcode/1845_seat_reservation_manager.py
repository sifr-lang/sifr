from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1845: Seat Reservation Manager
# Python version

class SeatManager:
    def __init__(self, n: int):
        self.seats = [i for i in range(1, n + 1)]
    def reserve(self) -> int:
        return heapq.heappop(self.seats)
    def unreserve(self, seatNumber: int) -> None:
        heapq.heappush(self.seats, seatNumber)

def main():
    obj = SeatManager(5)
    assert obj.reserve() == 1
    assert obj.reserve() == 2
    obj.unreserve(2)
    assert obj.reserve() == 2
    assert obj.reserve() == 3
    assert obj.reserve() == 4
    assert obj.reserve() == 5
    obj.unreserve(5)

if __name__ == "__main__":
    main()
