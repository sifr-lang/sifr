# LeetCode 846: Hand Of Straights
# Python version

def isNStraightHand(hand: List[int], groupSize: int) -> bool:
    if len(hand) % groupSize:
        return False

    count = {}
    for n in hand:
        count[n] = 1 + count.get(n, 0)

    minH = list(count.keys())
    heapq.heapify(minH)
    while minH:
        first = minH[0]
        for i in range(first, first + groupSize):
            if i not in count:
                return False
            count[i] -= 1
            if count[i] == 0:
                if i != minH[0]:
                    return False
                heapq.heappop(minH)
    return True



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
