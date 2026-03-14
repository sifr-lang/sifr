
# LeetCode 763: Partition Labels
# Python version

def partitionLabels(S: str) -> list[int]:
    count = {}
    res = []
    i, length = 0, len(S)
    for j in range(length):
        c = S[j]
        count[c] = j

    curLen = 0
    goal = 0
    while i < length:
        c = S[i]
        goal = max(goal, count[c])
        curLen += 1

        if goal == i:
            res.append(curLen)
            curLen = 0
        i += 1
    return res



def main():
    assert partitionLabels("ababcbacadefegdehijhklij") == [9, 7, 8]

if __name__ == "__main__":
    main()
