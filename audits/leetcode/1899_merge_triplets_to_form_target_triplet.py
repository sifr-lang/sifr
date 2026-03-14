
# LeetCode 1899: Merge Triplets To Form Target Triplet
# Python version

def mergeTriplets(triplets: list[list[int]], target: list[int]) -> bool:
    good = set()

    for t in triplets:
        if t[0] > target[0] or t[1] > target[1] or t[2] > target[2]:
            continue
        for i, v in enumerate(t):
            if v == target[i]:
                good.add(i)
    return len(good) == 3



def main():
    assert mergeTriplets([[2,5,3],[1,8,4],[1,7,5]], [2,7,5]) == True

if __name__ == "__main__":
    main()
