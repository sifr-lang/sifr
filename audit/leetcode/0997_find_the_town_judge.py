# LeetCode 997: Find The Town Judge
# Python version

def findJudge(n: int, trust: List[List[int]]) -> int:
    delta = defaultdict(int)

    for src, dst in trust:
        delta[src] -= 1
        delta[dst] += 1

    for i in range(1, n + 1):
        if delta[i] == n - 1:
            return i
    return -1



def main():
    print(findJudge(2, [[1,2]]))
    print(findJudge(3, [[1,3],[2,3]]))
    print(findJudge(3, [[1,3],[2,3],[3,1]]))

if __name__ == "__main__":
    main()
