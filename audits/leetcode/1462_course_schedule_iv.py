# LeetCode 1462: Course Schedule Iv
# Python version

def checkIfPrerequisite(numCourses: int, prerequisites: List[List[int]], queries: List[List[int]]) -> List[bool]:
    adj = defaultdict(list)
    for prereq, crs in prerequisites:
        adj[crs].append(prereq)
    
    def dfs(crs):
        if crs not in prereqMap:
            prereqMap[crs] = set()
            for pre in adj[crs]:
                prereqMap[crs] |= dfs(pre)
        prereqMap[crs].add(crs)
        return prereqMap[crs]

    prereqMap = {} # map course -> set indirect prereqs
    for crs in range(numCourses):
        dfs(crs)

    res = []
    for u, v in queries:
        res.append(u in prereqMap[v])
    return res



def main():
    assert checkIfPrerequisite(2, [[1, 0]], [[0, 1], [1, 0]]) == [False, True]
    assert checkIfPrerequisite(2, [], [[1, 0], [0, 1]]) == [False, False]
    assert checkIfPrerequisite(3, [[1, 2], [1, 0], [2, 0]], [[1, 0], [1, 2]]) == [True, True]

if __name__ == "__main__":
    main()
