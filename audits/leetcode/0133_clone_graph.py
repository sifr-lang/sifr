from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 133: Clone Graph
# Python version

def build_graph(adjacency: list[list[int]]) -> Node | None:
    if len(adjacency) == 0:
        return None
    nodes = [Node(i + 1) for i in range(len(adjacency))]
    for i, neighbors in enumerate(adjacency):
        nodes[i].neighbors = [nodes[value - 1] for value in neighbors]
    return nodes[0]


def graph_to_adj(node: Node | None) -> list[list[int]]:
    if node is None:
        return []
    queue = [node]
    seen = {node}
    by_val = {}
    while queue:
        cur = queue.pop(0)
        by_val[cur.val] = sorted(neighbor.val for neighbor in cur.neighbors)
        for neighbor in cur.neighbors:
            if neighbor not in seen:
                seen.add(neighbor)
                queue.append(neighbor)
    return [by_val[i] for i in range(1, len(by_val) + 1)]

class Node:
    def __init__(
        self,
        val: int = 0,
        next: 'Node | None' = None,
        random: 'Node | None' = None,
        left: 'Node | None' = None,
        right: 'Node | None' = None,
        neighbors: list['Node'] | None = None,
        key: int = -1,
    ):
        self.val = val
        self.next = next
        self.random = random
        self.left = left
        self.right = right
        self.neighbors = [] if neighbors is None else neighbors
        self.key = key

def cloneGraph(node: "Node") -> "Node":
    oldToNew = {}

    def dfs(node):
        if node in oldToNew:
            return oldToNew[node]

        copy = Node(node.val)
        oldToNew[node] = copy
        for nei in node.neighbors:
            copy.neighbors.append(dfs(nei))
        return copy

    return dfs(node) if node else None



def main():
    assert graph_to_adj(cloneGraph(build_graph([[2, 4], [1, 3], [2, 4], [1, 3]]))) == [[2, 4], [1, 3], [2, 4], [1, 3]]
    assert graph_to_adj(cloneGraph(build_graph([[]]))) == [[]]
    assert graph_to_adj(cloneGraph(build_graph([]))) == []

if __name__ == "__main__":
    main()