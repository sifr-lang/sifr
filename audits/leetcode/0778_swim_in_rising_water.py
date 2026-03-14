import heapq

# LeetCode 778: Swim In Rising Water
# Python version

def swimInWater(grid: list[list[int]]) -> int:
    N = len(grid)
    visit = set()
    minH = [[grid[0][0], 0, 0]]  # (time/max-height, r, c)
    directions = [[0, 1], [0, -1], [1, 0], [-1, 0]]

    visit.add((0, 0))
    while minH:
        t, r, c = heapq.heappop(minH)
        if r == N - 1 and c == N - 1:
            return t
        for dr, dc in directions:
            neiR, neiC = r + dr, c + dc
            if (
                neiR < 0
                or neiC < 0
                or neiR == N
                or neiC == N
                or (neiR, neiC) in visit
            ):
                continue
            visit.add((neiR, neiC))
            heapq.heappush(minH, [max(t, grid[neiR][neiC]), neiR, neiC])



def main():
    assert swimInWater([[0, 2], [1, 3]]) == 3
    assert swimInWater([[0, 1, 2, 3, 4], [24, 23, 22, 21, 5], [12, 13, 14, 15, 16], [11, 17, 18, 19, 20], [10, 9, 8, 7, 6]]) == 16

if __name__ == "__main__":
    main()
