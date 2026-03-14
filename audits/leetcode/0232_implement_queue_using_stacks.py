from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 232: Implement Queue Using Stacks
# Python version

class MyQueue:
    def __init__(self):
        self.append_stack = []
        self.inverted_stack = []
    def push(self, x: int) -> None:
        self.append_stack.append(x)
    def pop(self) -> int:
        if not self.inverted_stack:
            while self.append_stack:
                self.inverted_stack.append(self.append_stack.pop())
        return self.inverted_stack.pop()
    def peek(self) -> int:
        if not self.inverted_stack:
            while self.append_stack:
                self.inverted_stack.append(self.append_stack.pop())
        return self.inverted_stack[-1]
    def empty(self) -> bool:
        return not (self.append_stack or self.inverted_stack)

def main():
    obj = MyQueue()
    obj.push(1)
    obj.push(2)
    assert obj.peek() == 1
    assert obj.pop() == 1
    assert obj.empty() == False

if __name__ == "__main__":
    main()
