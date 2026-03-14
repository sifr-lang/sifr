import collections

# LeetCode 994: Rotting Oranges
# Python version

def orangesRotting(grid: list[list[int]]) -> int:
    q = collections.deque()
    fresh = 0
    time = 0

    for r in range(len(grid)):
        for c in range(len(grid[0])):
            if grid[r][c] == 1:
                fresh += 1
            if grid[r][c] == 2:
                q.append((r, c))

    directions = [[0, 1], [0, -1], [1, 0], [-1, 0]]
    while fresh > 0 and q:
        length = len(q)
        for i in range(length):
            r, c = q.popleft()

            for dr, dc in directions:
                row, col = r + dr, c + dc
                # if in bounds and nonrotten, make rotten
                # and add to q
                if (
                    row in range(len(grid))
                    and col in range(len(grid[0]))
                    and grid[row][col] == 1
                ):
                    grid[row][col] = 2
                    q.append((row, col))
                    fresh -= 1
        time += 1
    return time if fresh == 0 else -1



def main():
    assert orangesRotting([[2, 1, 1], [1, 1, 0], [0, 1, 1]]) == 4
    assert orangesRotting([[2, 1, 1], [0, 1, 1], [1, 0, 1]]) == -1
    assert orangesRotting([[0, 2]]) == 0

if __name__ == "__main__":
    main()
