# LeetCode 743: Network Delay Time
# Python version

def networkDelayTime(times: List[List[int]], n: int, k: int) -> int:
    edges = collections.defaultdict(list)
    for u, v, w in times:
        edges[u].append((v, w))

    minHeap = [(0, k)]
    visit = set()
    t = 0
    while minHeap:
        w1, n1 = heapq.heappop(minHeap)
        if n1 in visit:
            continue
        visit.add(n1)
        t = w1

        for n2, w2 in edges[n1]:
            if n2 not in visit:
                heapq.heappush(minHeap, (w1 + w2, n2))
    return t if len(visit) == n else -1

    # O(E * logV)



def main():
    assert networkDelayTime([[2, 1, 1], [2, 3, 1], [3, 4, 1]], 4, 2) == 2
    assert networkDelayTime([[1, 2, 1]], 2, 1) == 1
    assert networkDelayTime([[1, 2, 1]], 2, 2) == -1

if __name__ == "__main__":
    main()
