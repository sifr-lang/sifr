# LeetCode 56: Merge Intervals
# Python version

def merge(intervals: List[List[int]]) -> List[List[int]]:
    intervals.sort(key=lambda pair: pair[0])
    output = [intervals[0]]

    for start, end in intervals:
        lastEnd = output[-1][1]

        if start <= lastEnd:
            # merge
            output[-1][1] = max(lastEnd, end)
        else:
            output.append([start, end])
    return output



def main():
    print(merge([[1,3],[2,6],[8,10],[15,18]]))

if __name__ == "__main__":
    main()
