
# LeetCode 1343: Number Of Sub Arrays Of Size K And Average Greater Than Or Equal To Threshold
# Python version

def numOfSubarrays(arr: list[int], k: int, threshold: int) -> int:
    res = 0
    curSum = sum(arr[:k-1])

    for L in range(len(arr) - k + 1):
        curSum += arr[L + k - 1]
        if (curSum / k) >= threshold:
            res += 1
        curSum -= arr[L]
    return res



def main():
    assert numOfSubarrays([2,1,5,6,0,9,8], 3, 4) == 3

if __name__ == "__main__":
    main()
