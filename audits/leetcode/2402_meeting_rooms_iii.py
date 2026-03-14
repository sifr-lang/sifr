# LeetCode 2402: Meeting Rooms Iii
# Python version

def mostBooked(n: int, meetings: List[List[int]]) -> int:
    meetings.sort()

    available = [i for i in range(n)]
    used = []
    count = [0] * n

    for start, end in meetings:
        while used and start >= used[0][0]:
            _, room = heapq.heappop(used)
            heapq.heappush(available, room)

        if not available:
            end_time, room = heapq.heappop(used)
            end = end_time + (end - start)
            heapq.heappush(available, room)

        room = heapq.heappop(available)
        heapq.heappush(used, (end, room))
        count[room] += 1

    return count.index(max(count))



def main():
    assert mostBooked(2, [[0, 10], [1, 5], [2, 7], [3, 4]]) == 0
    assert mostBooked(3, [[1, 20], [2, 10], [3, 5], [4, 9], [6, 8]]) == 1

if __name__ == "__main__":
    main()
