
# LeetCode 274: H Index
# Python version

def hIndex(citations: list[int]) -> int:
    length = len(citations)
    citations.sort()
    for i in range(length):
        if citations[i] >= length - i:
            return length - i
    return 0



def main():
    assert hIndex([3,0,6,1,5]) == 3

if __name__ == "__main__":
    main()
