# LeetCode 207: Course Schedule
# Python version

def canFinish(numCourses: int, prerequisites: List[List[int]]) -> bool:
    # dfs
    preMap = {i: [] for i in range(numCourses)}

    # map each course to : prereq list
    for crs, pre in prerequisites:
        preMap[crs].append(pre)

    visiting = set()

    def dfs(crs):
        if crs in visiting:
            return False
        if preMap[crs] == []:
            return True

        visiting.add(crs)
        for pre in preMap[crs]:
            if not dfs(pre):
                return False
        visiting.remove(crs)
        preMap[crs] = []
        return True

    for c in range(numCourses):
        if not dfs(c):
            return False
    return True



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
