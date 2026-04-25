
# LeetCode 1489: Find Critical And Pseudo Critical Edges In Minimum Spanning Tree
# Python version

from helpers.dsu import UnionFind

def findCriticalAndPseudoCriticalEdges(n: int, edges: list[list[int]]) -> list[list[int]]:
    # Time: O(E^2) - UF operations are assumed to be approx O(1)
    for i, e in enumerate(edges):
        e.append(i) # [v1, v2, weight, original_index]

    edges.sort(key=lambda e: e[2])

    mst_weight = 0
    uf = UnionFind(n)
    for v1, v2, w, i in edges:
        if uf.union(v1, v2):
            mst_weight += w

    critical, pseudo = [], []
    for n1, n2, e_weight, i in edges:
        # Try without curr edge
        weight = 0
        uf = UnionFind(n)
        for v1, v2, w, j in edges:
            if i != j and uf.union(v1, v2):
                weight += w
        if max(uf.rank) != n or weight > mst_weight:
            critical.append(i)
            continue
        
        # Try with curr edge
        uf = UnionFind(n)
        uf.union(n1, n2)
        weight = e_weight
        for v1, v2, w, j in edges:
            if uf.union(v1, v2):
                weight += w
        if weight == mst_weight:
            pseudo.append(i)
    return [critical, pseudo]
def main():
    assert findCriticalAndPseudoCriticalEdges(5, [[0, 1, 1], [1, 2, 1], [2, 3, 2], [0, 3, 2], [0, 4, 3], [3, 4, 3], [1, 4, 6]]) == [[0, 1], [2, 3, 4, 5]]
    assert findCriticalAndPseudoCriticalEdges(4, [[0, 1, 1], [1, 2, 1], [2, 3, 1], [0, 3, 1]]) == [[], [0, 1, 2, 3]]

if __name__ == "__main__":
    main()
