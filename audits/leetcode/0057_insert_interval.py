# LeetCode 57: Insert Interval
# Python version

def insert(
    self, intervals: List[List[int]], newInterval: List[int]
) -> List[List[int]]:
    res = []

    for i in range(len(intervals)):
        if newInterval[1] < intervals[i][0]:
            res.append(newInterval)
            return res + intervals[i:]
        elif newInterval[0] > intervals[i][1]:
            res.append(intervals[i])
        else:
            newInterval = [
                min(newInterval[0], intervals[i][0]),
                max(newInterval[1], intervals[i][1]),
            ]
    res.append(newInterval)
    return res



def main():
    assert insert(None, [[1, 3], [6, 9]], [2, 5]) == [[1, 5], [6, 9]]
    assert insert(
        None,
        [[1, 2], [3, 5], [6, 7], [8, 10], [12, 16]],
        [4, 8],
    ) == [[1, 2], [3, 10], [12, 16]]

if __name__ == "__main__":
    main()
