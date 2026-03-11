# LeetCode 1029: Two City Scheduling
# Python version

def twoCitySchedCost(costs: List[List[int]]) -> int:
    diffs = []
    for c1, c2 in costs:
        diffs.append([c2 - c1, c1, c2])
    diffs.sort()
    res = 0
    for i in range(len(diffs)):
        if i < len(diffs) / 2:
            res += diffs[i][2]
        else:
            res += diffs[i][1]
    return res


def main():
    print(twoCitySchedCost([[10,20],[30,200],[400,50],[30,20]]))

if __name__ == "__main__":
    main()
