
# LeetCode 802: Find Eventual Safe States
# Python version

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

def eventualSafeNodes(graph: list[list[int]]) -> list[int]:
    n = len(graph)
    safe = {}
    res = []
    def dfs(i):
        if i in safe:
            return safe[i]
        safe[i] = False
        for nei in graph[i]:
            if not dfs(nei):
                return safe[i]
        safe[i] = True
        return safe[i]
    for i in range(len(graph)):
        if dfs(i): res.append(i)
    return res



def main():
    assert eventualSafeNodes([[1, 2], [2, 3], [5], [0], [5], [], []]) == [2, 4, 5, 6]
    assert eventualSafeNodes([[1, 2, 3, 4], [1, 2], [3, 4], [0, 4], []]) == [4]

if __name__ == "__main__":
    main()