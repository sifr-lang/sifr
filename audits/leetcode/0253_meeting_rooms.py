# LeetCode 253: Meeting Rooms
# Python version

"""
@param intervals: an array of meeting time intervals
@return: the minimum number of conference rooms required
"""


def minMeetingRooms(intervals):
    start = sorted([i[0] for i in intervals])
    end = sorted([i[1] for i in intervals])

    res, count = 0, 0
    s, e = 0, 0
    while s < len(intervals):
        if start[s] < end[e]:
            s += 1
            count += 1
        else:
            e += 1
            count -= 1
        res = max(res, count)
    return res



def main():
    assert minMeetingRooms([[0, 30], [5, 10], [15, 20]]) == 2
    assert minMeetingRooms([[7, 10], [2, 4]]) == 1

if __name__ == "__main__":
    main()
