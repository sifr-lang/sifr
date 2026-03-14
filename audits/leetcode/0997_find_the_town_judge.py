from collections import defaultdict

# LeetCode 997: Find The Town Judge
# Python version

def findJudge(n: int, trust: list[list[int]]) -> int:
    delta = defaultdict(int)

    for src, dst in trust:
        delta[src] -= 1
        delta[dst] += 1

    for i in range(1, n + 1):
        if delta[i] == n - 1:
            return i
    return -1



def main():
    assert findJudge(2, [[1,2]]) == 2
    assert findJudge(3, [[1,3],[2,3]]) == 3
    assert findJudge(3, [[1,3],[2,3],[3,1]]) == -1

if __name__ == "__main__":
    main()
