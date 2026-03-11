# LeetCode 802: Find Eventual Safe States
# Python version

def eventualSafeNodes(graph: List[List[int]]) -> List[int]:
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
    print("no test cases")

if __name__ == "__main__":
    main()
