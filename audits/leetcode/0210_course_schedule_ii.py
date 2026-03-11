# LeetCode 210: Course Schedule Ii
# Python version

def findOrder(numCourses: int, prerequisites: List[List[int]]) -> List[int]:
    prereq = {c: [] for c in range(numCourses)}
    for crs, pre in prerequisites:
        prereq[crs].append(pre)

    output = []
    visit, cycle = set(), set()

    def dfs(crs):
        if crs in cycle:
            return False
        if crs in visit:
            return True

        cycle.add(crs)
        for pre in prereq[crs]:
            if dfs(pre) == False:
                return False
        cycle.remove(crs)
        visit.add(crs)
        output.append(crs)
        return True

    for c in range(numCourses):
        if dfs(c) == False:
            return []
    return output



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
