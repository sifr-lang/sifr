
# LeetCode 323: Number Of Connected Components In An Undirected Graph
# Python version

from helpers.dsu import UnionFind

def countComponents(n: int, edges: list[list[int]]) -> int:
    dsu = UnionFind()
    for a, b in edges:
        dsu.union(a, b)
    return len(set(dsu.findParent(x) for x in range(n)))
def main():
    assert countComponents(5, [[0, 1], [1, 2], [3, 4]]) == 2
    assert countComponents(5, [[0, 1], [1, 2], [2, 3], [3, 4]]) == 1

if __name__ == "__main__":
    main()
