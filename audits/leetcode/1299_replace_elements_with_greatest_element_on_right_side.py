# LeetCode 1299: Replace Elements With Greatest Element On Right Side
# Python version

def replaceElements(arr: List[int]) -> List[int]:
    rightMax = -1
    for i in range(len(arr) -1, -1, -1):
        newMax = max(rightMax, arr[i])
        arr[i] = rightMax
        rightMax = newMax
    return arr



def main():
    assert replaceElements([17,18,5,4,6,1]) == [18, 6, 6, 6, 1, -1]

if __name__ == "__main__":
    main()
