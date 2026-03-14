from collections import deque

# LeetCode 1091: Shortest Path In Binary Matrix
# Python version

def shortestPathBinaryMatrix(grid: list[list[int]]) -> int:
    N = len(grid)
    q = deque([(0, 0, 1)]) # r, c, length
    visit = set((0, 0))
    direct = [[0, 1], [1, 0], [0, -1], [-1, 0],
              [1, 1], [-1, -1], [1, -1], [-1, 1]]
    while q:
        r, c, length = q.popleft()
        if (min(r, c) < 0 or max(r, c) >= N or
            grid[r][c]):
            continue
        if r == N - 1 and c == N - 1:
            return length
        for dr, dc in direct:
            if (r + dr, c + dc) not in visit:
                q.append((r + dr, c + dc, length + 1))
                visit.add((r + dr, c + dc))
    return -1



def main():
    assert shortestPathBinaryMatrix([[0, 1], [1, 0]]) == 2
    assert shortestPathBinaryMatrix([[0, 0, 0], [1, 1, 0], [1, 1, 0]]) == 4
    assert shortestPathBinaryMatrix([[1, 0, 0], [1, 1, 0], [1, 1, 0]]) == -1

if __name__ == "__main__":
    main()
